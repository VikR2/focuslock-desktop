//! Windows notifications (toasts) for Warn. A toast shows over the app you
//! switched to without taking focus from it.

use windows::core::HSTRING;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

const AUMID: &str = "FocusLock.App";
const ICON_PNG: &[u8] = include_bytes!("../../../assets/focuslock.png");

/// Register FocusLock as a notification sender for this user. Unpackaged apps
/// need an AppUserModelID under HKCU before Windows will show their toasts.
pub fn register() {
    let dir = std::env::var_os("APPDATA").map(std::path::PathBuf::from).map(|d| d.join("FocusLock"));
    let icon = dir.and_then(|d| {
        std::fs::create_dir_all(&d).ok()?;
        let path = d.join("icon.png");
        std::fs::write(&path, ICON_PNG).ok()?;
        Some(path)
    });
    if let Ok(key) = windows_registry::CURRENT_USER.create(format!(r"Software\Classes\AppUserModelId\{AUMID}")) {
        let _ = key.set_string("DisplayName", "FocusLock");
        if let Some(icon) = icon {
            let _ = key.set_string("IconUri", icon.to_string_lossy());
        }
    }
    unsafe {
        let _ = windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(AUMID));
    }
}

/// Show a two-line toast. The calling thread must have COM initialized.
pub fn toast(title: &str, body: &str) -> windows::core::Result<()> {
    let xml = format!(
        r#"<toast><visual><binding template="ToastGeneric"><text>{}</text><text>{}</text></binding></visual></toast>"#,
        escape(title),
        escape(body)
    );
    let doc = XmlDocument::new()?;
    doc.LoadXml(&HSTRING::from(xml))?;
    let notification = ToastNotification::CreateToastNotification(&doc)?;
    ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?.Show(&notification)
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    /// Shows a real toast; run by hand with `cargo test -- --ignored shows_a_real_toast`.
    #[test]
    #[ignore]
    fn shows_a_real_toast() {
        crate::engine::win::init_com_for_thread();
        super::register();
        super::toast("Character Map is open", "You set it to warn during focus. 23 min left.").expect("toast shown");
    }

    #[test]
    fn escapes_xml() {
        assert_eq!(super::escape(r#"A&B <"C">"#), "A&amp;B &lt;&quot;C&quot;&gt;");
    }
}
