//! The app catalog: every launchable or running app, one entry per executable.

use super::{app_key, exe_key, is_protected_exe};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Where to ask the Windows shell for an app's icon.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IconSource {
    /// A file the shell can render: a shortcut (.lnk) or an executable.
    Path(String),
    /// A Store app, addressed as `shell:AppsFolder\<AUMID>`.
    AppsFolder(String),
}

/// How an app was found. Lower is more trustworthy for its name and icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    StartMenu,
    Store,
    Uninstall,
    Running,
}

/// One raw sighting of an app from one source.
#[derive(Clone, Debug)]
pub struct Sighting {
    pub name: String,
    pub exe_path: String,
    pub icon: IconSource,
    pub origin: Origin,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppEntry {
    /// `discord.exe`, or `pkg:<family>` for Store apps. Rules match on this.
    pub key: String,
    pub name: String,
    pub exe_path: String,
    pub icon: IconSource,
    pub running: bool,
}

/// Collapse sightings into one entry per executable, keeping the best name and icon.
pub fn merge(sightings: Vec<Sighting>) -> Vec<AppEntry> {
    let mut best: HashMap<String, (Origin, AppEntry)> = HashMap::new();

    // Registry entries often point at a setup or helper exe of an app that is
    // already listed from the Start menu or the Store under the same name.
    let trusted_names: std::collections::HashSet<String> = sightings
        .iter()
        .filter(|s| s.origin <= Origin::Store)
        .map(|s| s.name.trim().to_lowercase())
        .collect();

    for s in sightings {
        let key = app_key(&s.exe_path);
        let exe = exe_key(&s.exe_path);
        if key.is_empty() || is_protected_exe(&exe) || is_noise(&s.name, &exe) {
            continue;
        }
        let registry_only = s.origin == Origin::Uninstall;
        if registry_only && trusted_names.contains(&s.name.trim().to_lowercase()) && !best.contains_key(&key) {
            continue;
        }
        let running = s.origin == Origin::Running;
        let name = clean_name(&s.name, &key);
        match best.get_mut(&key) {
            Some((origin, entry)) => {
                entry.running |= running;
                if s.origin < *origin {
                    *origin = s.origin;
                    entry.name = name;
                    entry.icon = s.icon;
                    if !s.exe_path.is_empty() {
                        entry.exe_path = s.exe_path;
                    }
                }
            }
            None => {
                best.insert(
                    key.clone(),
                    (
                        s.origin,
                        AppEntry { key, name, exe_path: s.exe_path, icon: s.icon, running },
                    ),
                );
            }
        }
    }

    let mut apps: Vec<AppEntry> = best.into_values().map(|(_, e)| e).collect();
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.key.cmp(&b.key)));
    apps
}

/// Shortcuts and registry entries that are not apps a person would block.
fn is_noise(name: &str, key: &str) -> bool {
    let n = name.to_lowercase();
    const NAME_NOISE: &[&str] = &[
        "uninstall", "readme", "read me", "release notes", "website", "documentation",
        "license", "help and support", "safe mode", "updater", "crash handler",
        "crash reporter", "autostarter", "redistributable", "runtime", "reset ",
        " sdk", "driver", "debugger",
    ];
    const EXE_NOISE: &[&str] = &[
        "unins000.exe", "uninstall.exe", "uninst.exe", "setup.exe", "install.exe",
        "msiexec.exe", "rundll32.exe", "chrome_proxy.exe", "msedge_proxy.exe",
        "cmd.exe", "powershell.exe", "wscript.exe", "notepad.exe.mui",
    ];
    NAME_NOISE.iter().any(|w| n.contains(w))
        || EXE_NOISE.contains(&key)
        || key.contains("uninst")
        || key.ends_with("-setup.exe")
}

/// Shortcut names sometimes carry the extension or a trailing "(x64)".
fn clean_name(name: &str, key: &str) -> String {
    let mut n = name.trim().to_string();
    for suffix in [".lnk", ".exe", ".EXE"] {
        if let Some(stripped) = n.strip_suffix(suffix) {
            n = stripped.to_string();
        }
    }
    if n.is_empty() {
        n = key.trim_end_matches(".exe").to_string();
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(name: &str, path: &str, origin: Origin) -> Sighting {
        Sighting {
            name: name.into(),
            exe_path: path.into(),
            icon: IconSource::Path(path.into()),
            origin,
        }
    }

    #[test]
    fn one_entry_per_exe_and_start_menu_name_wins() {
        let apps = merge(vec![
            s("Discord.exe", r"C:\Users\me\AppData\Local\Discord\app-1.0.9\Discord.exe", Origin::Running),
            s("Discord", r"C:\Users\me\AppData\Local\Discord\app-1.0.9\Discord.exe", Origin::StartMenu),
        ]);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].key, "discord.exe");
        assert_eq!(apps[0].name, "Discord");
        assert!(apps[0].running);
    }

    #[test]
    fn versioned_store_paths_collapse_to_one_app() {
        let apps = merge(vec![
            s("TradingView", r"C:\Program Files\WindowsApps\TradingView.Desktop_3.1.0.7818_x64__n534\TradingView.exe", Origin::Store),
            s("TradingView", r"C:\Program Files\WindowsApps\TradingView.Desktop_3.5.1.8500_x64__n534\TradingView.exe", Origin::Store),
        ]);
        assert_eq!(apps.len(), 1);
    }

    #[test]
    fn store_package_exes_merge_into_the_store_entry() {
        let apps = merge(vec![
            Sighting {
                name: "Spotify".into(),
                exe_path: r"C:\Program Files\WindowsApps\SpotifyAB.SpotifyMusic_1.278.418.0_x64__zpdnekdrzrea0\SpotifyMigrator.exe".into(),
                icon: IconSource::AppsFolder("SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify".into()),
                origin: Origin::Store,
            },
            s("Spotify", r"C:\Program Files\WindowsApps\SpotifyAB.SpotifyMusic_1.278.418.0_x64__zpdnekdrzrea0\Spotify.exe", Origin::Running),
        ]);
        assert_eq!(apps.len(), 1);
        assert!(apps[0].running);
        assert!(matches!(apps[0].icon, IconSource::AppsFolder(_)));
    }

    #[test]
    fn registry_setup_exes_of_listed_apps_are_dropped() {
        let apps = merge(vec![
            s("3DMark", r"C:\3DMark\3DMark.exe", Origin::StartMenu),
            s("3DMark", r"C:\3DMark\3DMarkInstaller.exe", Origin::Uninstall),
        ]);
        assert_eq!(apps.len(), 1);
    }

    #[test]
    fn uninstallers_and_system_processes_are_dropped() {
        let apps = merge(vec![
            s("Uninstall Steam", r"C:\Steam\uninstall.exe", Origin::StartMenu),
            s("Windows Explorer", r"C:\Windows\explorer.exe", Origin::Running),
            s("Steam", r"C:\Steam\steam.exe", Origin::StartMenu),
        ]);
        assert_eq!(apps.iter().map(|a| a.key.as_str()).collect::<Vec<_>>(), vec!["steam.exe"]);
    }
}
