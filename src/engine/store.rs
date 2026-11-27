//! Saved state: one small JSON file in %APPDATA%\FocusLock.

use super::session::Session;
use serde::{Deserialize, Serialize};
use super::catalog::AppEntry;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Close the app as soon as it opens.
    #[default]
    Lock,
    /// Let it open, but show a warning.
    Warn,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// Lowercase executable name, e.g. `spotify.exe`.
    pub key: String,
    pub name: String,
    pub mode: Mode,
    /// Temporarily allowed until this time (unix seconds).
    #[serde(default)]
    pub allowed_until: Option<u64>,
}

impl Rule {
    pub fn is_allowed(&self, now: u64) -> bool {
        self.allowed_until.is_some_and(|t| now < t)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub session_minutes: u64,
    /// Seconds to wait before an app can be allowed during a session.
    pub unlock_wait_secs: u64,
    /// Minutes an app stays allowed after unlocking it.
    pub unlock_minutes: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { session_minutes: 25, unlock_wait_secs: 30, unlock_minutes: 10 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupApp {
    pub key: String,
    pub name: String,
}

/// A focus group: the apps you keep while everything else stays closed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub apps: Vec<GroupApp>,
}

impl Group {
    pub fn has(&self, key: &str) -> bool {
        self.apps.iter().any(|a| a.key == key)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub rules: Vec<Rule>,
    pub settings: Settings,
    pub session: Option<Session>,
    pub groups: Vec<Group>,
}

impl State {
    pub fn rule(&self, key: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.key == key)
    }
}

/// `%APPDATA%\FocusLock\state.json`, or `FOCUSLOCK_STATE` when set (used for testing).
pub fn state_path() -> PathBuf {
    if let Some(p) = std::env::var_os("FOCUSLOCK_STATE") {
        return PathBuf::from(p);
    }
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    base.join("FocusLock").join("state.json")
}

/// The app list from the last scan, next to the state file. Read at launch so
/// the list shows at once; a fresh scan replaces it in the background.
pub fn apps_cache_path(state: &Path) -> PathBuf {
    state.with_file_name("apps.json")
}

pub fn load_apps(path: &Path) -> Vec<AppEntry> {
    std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn save_apps(path: &Path, apps: &[AppEntry]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(apps)?)?;
    std::fs::rename(&tmp, path)
}

/// Load saved state. A missing file is a fresh install; a corrupt one is kept aside.
pub fn load(path: &PathBuf) -> State {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| {
            let _ = std::fs::rename(path, path.with_extension("json.bad"));
            State::default()
        }),
        Err(_) => State::default(),
    }
}

/// Write via a temp file and rename, so a crash never leaves half a file.
pub fn save(path: &PathBuf, state: &State) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(state)?)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_tolerates_missing_fields() {
        let dir = std::env::temp_dir().join(format!("focuslock-test-{}", std::process::id()));
        let path = dir.join("state.json");
        let mut s = State::default();
        s.rules.push(Rule { key: "spotify.exe".into(), name: "Spotify".into(), mode: Mode::Lock, allowed_until: None });
        save(&path, &s).unwrap();
        assert_eq!(load(&path), s);

        std::fs::write(&path, r#"{"rules":[{"key":"a.exe","name":"A","mode":"warn"}]}"#).unwrap();
        let loaded = load(&path);
        assert_eq!(loaded.rules[0].mode, Mode::Warn);
        assert_eq!(loaded.settings, Settings::default());

        std::fs::write(&path, "not json").unwrap();
        assert_eq!(load(&path), State::default());
        assert!(path.with_extension("json.bad").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn app_cache_round_trips_and_missing_is_empty() {
        use crate::engine::catalog::IconSource;
        let dir = std::env::temp_dir().join(format!("focuslock-apps-{}", std::process::id()));
        let path = apps_cache_path(&dir.join("state.json"));
        assert!(load_apps(&path).is_empty());
        let apps = vec![AppEntry {
            key: "pkg:spotifyab.spotifymusic_zpdnekdrzrea0".into(),
            name: "Spotify".into(),
            exe_path: String::new(),
            icon: IconSource::AppsFolder("SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify".into()),
            running: false,
        }];
        save_apps(&path, &apps).unwrap();
        assert_eq!(load_apps(&path), apps);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn allowance_expires() {
        let r = Rule { key: "a.exe".into(), name: "A".into(), mode: Mode::Lock, allowed_until: Some(100) };
        assert!(r.is_allowed(99));
        assert!(!r.is_allowed(100));
    }
}
