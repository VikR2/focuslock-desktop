//! Everything that is not UI: the app catalog, search, the blocker and saved state.

pub mod blocker;
pub mod catalog;
pub mod search;
pub mod session;
pub mod store;

#[cfg(windows)]
pub mod win;

/// A process or app key: the lowercase executable file name, e.g. `spotify.exe`.
/// Rules match on this so they survive app updates that change the install path.
pub fn exe_key(name_or_path: &str) -> String {
    let trimmed = name_or_path.trim().trim_matches('"');
    let file = trimmed
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(trimmed)
        .to_ascii_lowercase();
    if file.ends_with(".exe") || file.is_empty() {
        file
    } else {
        format!("{file}.exe")
    }
}

/// The key an app is blocked by. Store apps run several exes from one package
/// folder (Spotify ships Spotify.exe and SpotifyMigrator.exe), so anything under
/// `WindowsApps` is keyed by its package family, which stays the same across updates:
/// `pkg:spotifyab.spotifymusic_zpdnekdrzrea0`. Everything else is keyed by exe name.
pub fn app_key(path: &str) -> String {
    package_family(path).map(|f| format!("pkg:{f}")).unwrap_or_else(|| exe_key(path))
}

/// `...\WindowsApps\SpotifyAB.SpotifyMusic_1.278.418.0_x64__zpdnekdrzrea0\Spotify.exe`
/// -> `spotifyab.spotifymusic_zpdnekdrzrea0` (package name + publisher id).
pub fn package_family(path: &str) -> Option<String> {
    let lower = path.to_ascii_lowercase().replace('/', "\\");
    let idx = lower.find("\\windowsapps\\")?;
    let dir = lower[idx + "\\windowsapps\\".len()..].split('\\').next()?;
    let parts: Vec<&str> = dir.split('_').collect();
    if parts.len() < 3 {
        return None;
    }
    Some(format!("{}_{}", parts[0], parts[parts.len() - 1]))
}

/// Does a running process belong to the app with this key?
pub fn process_matches(rule_key: &str, proc_exe_key: &str, proc_path: Option<&str>) -> bool {
    if rule_key.starts_with("pkg:") {
        proc_path.is_some_and(|p| app_key(p) == rule_key)
    } else {
        rule_key == proc_exe_key
    }
}

/// Files under the Windows folder: shell and system UI, never closed by a vault.
pub fn is_windows_path(path: &str) -> bool {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into()).to_ascii_lowercase();
    path.to_ascii_lowercase().starts_with(&(windir + "\\"))
}

/// Processes FocusLock must never terminate, whatever the rules say.
pub fn is_protected_exe(key: &str) -> bool {
    const PROTECTED: &[&str] = &[
        "system", "system.exe", "smss.exe", "csrss.exe", "wininit.exe", "winlogon.exe",
        "services.exe", "lsass.exe", "svchost.exe", "dwm.exe", "explorer.exe", "fontdrvhost.exe",
        "sihost.exe", "ctfmon.exe", "taskhostw.exe", "runtimebroker.exe", "searchhost.exe",
        "startmenuexperiencehost.exe", "shellexperiencehost.exe", "textinputhost.exe",
        "conhost.exe", "lockapp.exe", "logonui.exe", "securityhealthservice.exe",
        "msmpeng.exe", "audiodg.exe", "taskmgr.exe", "focuslock.exe",
    ];
    PROTECTED.contains(&key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_key_takes_file_name_and_lowercases() {
        assert_eq!(
            exe_key(r"C:\Program Files\WindowsApps\SpotifyAB.SpotifyMusic_1.278.418.0_x64__zpdnekdrzrea0\Spotify.exe"),
            "spotify.exe"
        );
        assert_eq!(exe_key("NinjaTrader"), "ninjatrader.exe");
        assert_eq!(exe_key("\"C:/x/Chrome.EXE\""), "chrome.exe");
    }

    #[test]
    fn store_apps_key_by_package_across_versions_and_exes() {
        let a = r"C:\Program Files\WindowsApps\SpotifyAB.SpotifyMusic_1.278.418.0_x64__zpdnekdrzrea0\Spotify.exe";
        let b = r"C:\Program Files\WindowsApps\SpotifyAB.SpotifyMusic_1.280.0.0_x64__zpdnekdrzrea0\SpotifyMigrator.exe";
        assert_eq!(app_key(a), "pkg:spotifyab.spotifymusic_zpdnekdrzrea0");
        assert_eq!(app_key(a), app_key(b));
        assert_eq!(app_key(r"C:\Steam\steam.exe"), "steam.exe");
        assert!(process_matches("pkg:spotifyab.spotifymusic_zpdnekdrzrea0", "spotify.exe", Some(a)));
        assert!(!process_matches("pkg:spotifyab.spotifymusic_zpdnekdrzrea0", "spotify.exe", None));
        assert!(process_matches("steam.exe", "steam.exe", None));
    }

    #[test]
    fn shell_processes_are_protected() {
        assert!(is_protected_exe("explorer.exe"));
        assert!(!is_protected_exe("discord.exe"));
    }
}
