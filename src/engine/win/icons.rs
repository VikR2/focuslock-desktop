//! App icons, asked from the Windows shell so they match Start and the taskbar.
//!
//! Shortcuts, executables and Store apps (`shell:AppsFolder\<AUMID>`) all go
//! through `IShellItemImageFactory`, which resolves custom shortcut icons,
//! Squirrel launchers and packaged app logos the same way Explorer does.

use crate::engine::catalog::IconSource;
use windows::core::HSTRING;
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
};

/// Straight-alpha BGRA pixels, top row first.
#[derive(Clone, Debug)]
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

/// Render an app's icon at about `size` pixels. The thread must have COM initialized.
pub fn icon(source: &IconSource, size: u32) -> Option<Bitmap> {
    let parsing_name = match source {
        IconSource::Path(p) if !p.is_empty() => p.clone(),
        IconSource::AppsFolder(aumid) => format!("shell:AppsFolder\\{aumid}"),
        _ => return None,
    };
    unsafe {
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(&HSTRING::from(parsing_name), None).ok()?;
        let side = size as i32;
        let hbitmap = factory
            .GetImage(SIZE { cx: side, cy: side }, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK)
            .ok()?;
        let result = read_bitmap(hbitmap);
        let _ = DeleteObject(HGDIOBJ(hbitmap.0));
        result
    }
}

unsafe fn read_bitmap(hbitmap: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<Bitmap> {
    let mut info = BITMAP::default();
    let got = GetObjectW(
        HGDIOBJ(hbitmap.0),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut info as *mut _ as *mut _),
    );
    if got == 0 || info.bmWidth <= 0 || info.bmHeight == 0 {
        return None;
    }
    let (w, h) = (info.bmWidth as u32, info.bmHeight.unsigned_abs());
    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w as i32,
            biHeight: -(h as i32), // negative: top-down rows
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    let dc = CreateCompatibleDC(None);
    let lines = GetDIBits(dc, hbitmap, 0, h, Some(pixels.as_mut_ptr() as *mut _), &mut header, DIB_RGB_COLORS);
    let _ = DeleteDC(dc);
    if lines == 0 {
        return None;
    }
    fix_alpha(&mut pixels);
    Some(Bitmap { width: w, height: h, bgra: pixels })
}

/// The shell hands back premultiplied alpha, or no alpha at all for old icons.
/// GPUI wants straight alpha.
fn fix_alpha(px: &mut [u8]) {
    if px.as_chunks::<4>().0.iter().all(|p| p[3] == 0) {
        for p in px.as_chunks_mut::<4>().0 {
            p[3] = 255;
        }
        return;
    }
    let premultiplied = px.as_chunks::<4>().0.iter().all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]);
    if !premultiplied {
        return;
    }
    for p in px.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        if a > 0 && a < 255 {
            for c in &mut p[..3] {
                *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_fallback_when_alpha_missing() {
        let mut px = vec![10, 20, 30, 0, 1, 2, 3, 0];
        fix_alpha(&mut px);
        assert_eq!(px, vec![10, 20, 30, 255, 1, 2, 3, 255]);
    }

    #[test]
    fn unpremultiplies_half_transparent_pixels() {
        let mut px = vec![50, 64, 0, 128];
        fix_alpha(&mut px);
        assert_eq!(px, vec![100, 128, 0, 128]);
    }

    #[test]
    fn explorer_icon_renders() {
        super::super::init_com_for_thread();
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        let bmp = icon(&IconSource::Path(format!(r"{windir}\explorer.exe")), 48).expect("explorer icon");
        assert!(bmp.width >= 32 && bmp.height >= 32);
        assert!(bmp.bgra.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
    }
}
