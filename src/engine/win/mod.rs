//! Windows-only pieces: finding apps, reading icons, watching and closing processes.

pub mod apps;
pub mod icons;
pub mod notify;
pub mod procs;

use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

/// Call once on any worker thread that uses the shell or WinRT.
pub fn init_com_for_thread() {
    // S_FALSE (already initialized) is fine; a mode clash only means another
    // owner initialized this thread, which still lets the shell calls run.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
}

pub(crate) fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// Expand `%VAR%` references, leaving unknown ones as they are.
pub(crate) fn expand_env(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let var = &after[..end];
                match std::env::var(var) {
                    Ok(v) if !var.is_empty() => out.push_str(&v),
                    _ => {
                        out.push('%');
                        out.push_str(var);
                        out.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_known_vars_only() {
        std::env::set_var("FOCUSLOCK_TEST_DIR", r"C:\x");
        assert_eq!(expand_env(r"%FOCUSLOCK_TEST_DIR%\a.exe"), r"C:\x\a.exe");
        assert_eq!(expand_env("%NOPE_NOT_SET%\\a"), "%NOPE_NOT_SET%\\a");
        assert_eq!(expand_env("100%"), "100%");
    }
}
