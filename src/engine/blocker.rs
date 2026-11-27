//! Enforces rules while a session runs. Lives on its own thread, so blocking
//! keeps working whatever the window is doing, and stops on its own when the
//! session ends.
//!
//! Lock closes every matching process. Warn fires when you switch to a warned
//! app (its window comes to the front), at most once a minute per app, and
//! shows a Windows notification. A focus-group session is a vault: every app
//! with a window that is not in the group is closed, and rules are set aside.

use super::session::{now_secs, Session};
use super::store::{Mode, Rule};
use super::{app_key, is_protected_exe, is_windows_path, process_matches};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Minimum gap between two warnings for the same app.
pub const WARN_EVERY_SECS: u64 = 60;

/// What the blocker needs to know. The UI replaces it on every change.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub rules: Vec<Rule>,
    pub session: Option<Session>,
    /// Set during a focus-group session: the app keys allowed to stay open.
    pub vault: Option<HashSet<String>>,
}

impl Plan {
    fn enforcing(&self, now: u64) -> bool {
        self.session.as_ref().is_some_and(|s| s.is_enforcing(now))
    }

    /// Rules in force. A vault session sets rules aside.
    fn active(&self, now: u64, mode: Mode) -> impl Iterator<Item = &Rule> {
        let vault = self.vault.is_some();
        self.rules.iter().filter(move |r| !vault && r.mode == mode && !r.is_allowed(now))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Closed { name: String, key: String },
    Warned { name: String },
    /// Windows refused to close it (usually an app running as administrator).
    CouldNotClose { name: String, key: String },
}

/// A running process as the blocker sees it.
pub struct Running<'a> {
    pub pid: u32,
    pub key: &'a str,
}

/// Processes to close: everything that matches a locked rule. `path_of` is only
/// called for Store-app rules, which match by package folder.
pub fn to_close(
    plan: &Plan,
    now: u64,
    procs: &[Running<'_>],
    mut path_of: impl FnMut(u32) -> Option<String>,
) -> Vec<(u32, String)> {
    if !plan.enforcing(now) {
        return Vec::new();
    }
    let locked: Vec<&Rule> = plan.active(now, Mode::Lock).collect();
    let own = std::process::id();
    let mut out = Vec::new();
    for p in procs {
        if p.pid == own || is_protected_exe(p.key) {
            continue;
        }
        let mut path: Option<Option<String>> = None;
        for r in &locked {
            let proc_path = if r.key.starts_with("pkg:") {
                path.get_or_insert_with(|| path_of(p.pid)).as_deref()
            } else {
                None
            };
            if process_matches(&r.key, p.key, proc_path) {
                out.push((p.pid, r.name.clone()));
                break;
            }
        }
    }
    out
}

/// Vault: apps with a visible window that are outside the group. Returns
/// `(pid, app key, exe name)`. Windows' own UI, protected processes, FocusLock and
/// apps unlocked for now are left alone; so is anything whose path can't be read.
pub fn vault_to_close(
    plan: &Plan,
    now: u64,
    procs: &[Running<'_>],
    windowed: &HashSet<u32>,
    mut path_of: impl FnMut(u32) -> Option<String>,
) -> Vec<(u32, String, String)> {
    let (Some(keep), Some(session)) = (&plan.vault, &plan.session) else {
        return Vec::new();
    };
    if !session.is_enforcing(now) {
        return Vec::new();
    }
    let own = std::process::id();
    let mut out = Vec::new();
    for p in procs {
        if p.pid == own || !windowed.contains(&p.pid) || is_protected_exe(p.key) {
            continue;
        }
        let Some(path) = path_of(p.pid) else { continue };
        if is_windows_path(&path) {
            continue;
        }
        let key = app_key(&path);
        if keep.contains(&key) || session.is_allowed(&key, now) {
            continue;
        }
        out.push((p.pid, key, p.key.trim_end_matches(".exe").to_string()));
    }
    out
}

/// Decides when to warn: when you switch into a warned app, not more than once
/// a minute per app. Staying in the app does not repeat the warning.
#[derive(Default)]
pub struct WarnTracker {
    in_front: Option<String>,
    last_warned: HashMap<String, u64>,
}

impl WarnTracker {
    /// `front` is the exe key and path of the foreground window's process.
    /// Returns the rule name to warn about, if this is the moment to warn.
    pub fn check(&mut self, plan: &Plan, now: u64, front: Option<(&str, Option<&str>)>) -> Option<String> {
        let rule = front.filter(|_| plan.enforcing(now)).and_then(|(key, path)| {
            plan.active(now, Mode::Warn).find(|r| process_matches(&r.key, key, path))
        });
        let Some(rule) = rule else {
            self.in_front = None;
            return None;
        };
        let switched_in = self.in_front.as_deref() != Some(rule.key.as_str());
        self.in_front = Some(rule.key.clone());
        let due = self.last_warned.get(&rule.key).is_none_or(|&t| now >= t + WARN_EVERY_SECS);
        if switched_in && due {
            self.last_warned.insert(rule.key.clone(), now);
            Some(rule.name.clone())
        } else {
            None
        }
    }
}

/// Start the blocker thread. Returns the shared plan to update and the event stream.
#[cfg(windows)]
pub fn spawn() -> (Arc<Mutex<Plan>>, Receiver<Event>) {
    let plan = Arc::new(Mutex::new(Plan::default()));
    let (tx, rx) = std::sync::mpsc::channel();
    let shared = plan.clone();
    std::thread::Builder::new()
        .name("focuslock-blocker".into())
        .spawn(move || run(shared, tx))
        .expect("start blocker thread");
    (plan, rx)
}

#[cfg(windows)]
fn run(plan: Arc<Mutex<Plan>>, tx: Sender<Event>) {
    use super::win::{notify, procs};
    super::win::init_com_for_thread();
    let mut warn = WarnTracker::default();
    loop {
        let snapshot = plan.lock().map(|p| p.clone()).unwrap_or_default();
        let now = now_secs();
        let mut events = Vec::new();
        if snapshot.enforcing(now) {
            let list = procs::list();
            let running: Vec<Running> = list.iter().map(|p| Running { pid: p.pid, key: &p.key }).collect();
            let mut closing = to_close(&snapshot, now, &running, procs::image_path)
                .into_iter()
                .map(|(pid, name)| {
                    let key = snapshot.rules.iter().find(|r| r.name == name).map(|r| r.key.clone()).unwrap_or_default();
                    (pid, key, name)
                })
                .collect::<Vec<_>>();
            if snapshot.vault.is_some() {
                closing = vault_to_close(&snapshot, now, &running, &procs::windowed_pids(), procs::image_path);
            }
            for (pid, key, name) in closing {
                events.push(if procs::kill(pid) { Event::Closed { name, key } } else { Event::CouldNotClose { name, key } });
            }
            let front = procs::foreground_pid().and_then(|pid| {
                list.iter().find(|p| p.pid == pid).map(|p| (p.key.clone(), procs::image_path(pid)))
            });
            if let Some(name) = warn.check(&snapshot, now, front.as_ref().map(|(k, p)| (k.as_str(), p.as_deref()))) {
                let minutes = snapshot.session.as_ref().map(|s| s.left(now).div_ceil(60)).unwrap_or(0);
                let _ = notify::toast(
                    &format!("{name} is open"),
                    &format!("You set it to warn during focus. {minutes} min left."),
                );
                events.push(Event::Warned { name });
            }
        } else {
            warn.check(&snapshot, now, None);
        }
        for event in events {
            if tx.send(event).is_err() {
                return; // the window is gone
            }
        }
        std::thread::sleep(Duration::from_millis(750));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(key: &str, mode: Mode) -> Rule {
        Rule { key: key.into(), name: key.into(), mode, allowed_until: None }
    }

    fn plan(rules: Vec<Rule>, session: Option<Session>) -> Plan {
        Plan { rules, session, vault: None }
    }

    fn vault(keys: &[&str], session: Session) -> Plan {
        Plan { rules: vec![rule("discord.exe", Mode::Lock)], session: Some(session), vault: Some(keys.iter().map(|k| k.to_string()).collect()) }
    }

    #[test]
    fn vault_closes_windowed_apps_outside_the_group() {
        let p = vault(&["tradingview.exe"], Session::start(0, 600, ""));
        let procs = [
            Running { pid: 1, key: "tradingview.exe" },
            Running { pid: 2, key: "discord.exe" },
            Running { pid: 3, key: "steamwebhelper.exe" },
            Running { pid: 4, key: "explorer.exe" },
            Running { pid: 5, key: "systemsettings.exe" },
        ];
        let windowed: HashSet<u32> = [1, 2, 4, 5].into();
        let path = |pid: u32| {
            Some(match pid {
                1 => r"C:\Apps\TradingView.exe".to_string(),
                2 => r"C:\Users\me\AppData\Local\Discord\app-1\Discord.exe".to_string(),
                4 => r"C:\Windows\explorer.exe".to_string(),
                _ => r"C:\Windows\ImmersiveControlPanel\SystemSettings.exe".to_string(),
            })
        };
        let closed = vault_to_close(&p, 5, &procs, &windowed, path);
        assert_eq!(closed, vec![(2, "discord.exe".to_string(), "discord".to_string())], "only Discord: no window for 3, Windows UI for 4 and 5");
    }

    #[test]
    fn vault_respects_unlocks_pause_and_sets_rules_aside() {
        let mut s = Session::start(0, 600, "");
        s.allowed.push(("discord.exe".into(), 100));
        let p = vault(&[], s.clone());
        let procs = [Running { pid: 2, key: "discord.exe" }];
        let windowed: HashSet<u32> = [2].into();
        let path = |_| Some(r"C:\D\Discord.exe".to_string());
        assert!(vault_to_close(&p, 50, &procs, &windowed, path).is_empty(), "unlocked");
        assert_eq!(vault_to_close(&p, 150, &procs, &windowed, path).len(), 1, "unlock expired");
        s.pause(10);
        assert!(vault_to_close(&vault(&[], s), 150, &procs, &windowed, path).is_empty(), "paused");
        // The Lock rule on discord.exe is not used while a vault runs.
        assert!(to_close(&vault(&[], Session::start(0, 600, "")), 5, &procs, |_| None).is_empty());
    }

    #[test]
    fn nothing_is_closed_without_a_running_session() {
        let procs = [Running { pid: 10, key: "discord.exe" }];
        let p = plan(vec![rule("discord.exe", Mode::Lock)], None);
        assert!(to_close(&p, 100, &procs, |_| None).is_empty());

        let mut paused = Session::start(0, 600, "");
        paused.pause(10);
        let p = plan(vec![rule("discord.exe", Mode::Lock)], Some(paused));
        assert!(to_close(&p, 100, &procs, |_| None).is_empty());

        let p = plan(vec![rule("discord.exe", Mode::Lock)], Some(Session::start(0, 60, "")));
        assert!(to_close(&p, 61, &procs, |_| None).is_empty(), "finished session");
    }

    #[test]
    fn lock_closes_and_warn_does_not() {
        let p = plan(
            vec![rule("discord.exe", Mode::Lock), rule("steam.exe", Mode::Warn)],
            Some(Session::start(0, 600, "")),
        );
        let procs = [
            Running { pid: 10, key: "discord.exe" },
            Running { pid: 11, key: "steam.exe" },
            Running { pid: 12, key: "code.exe" },
        ];
        assert_eq!(to_close(&p, 5, &procs, |_| None), vec![(10, "discord.exe".to_string())]);
    }

    #[test]
    fn store_rules_match_by_package_folder_and_unlocked_rules_are_skipped() {
        let spotify = "pkg:spotifyab.spotifymusic_zpdnekdrzrea0";
        let mut unlocked = rule("discord.exe", Mode::Lock);
        unlocked.allowed_until = Some(1_000);
        let p = plan(vec![rule(spotify, Mode::Lock), unlocked], Some(Session::start(0, 600, "")));
        let procs = [Running { pid: 20, key: "spotify.exe" }, Running { pid: 21, key: "discord.exe" }];
        let closed = to_close(&p, 5, &procs, |pid| {
            (pid == 20).then(|| {
                r"C:\Program Files\WindowsApps\SpotifyAB.SpotifyMusic_1.280.0.0_x64__zpdnekdrzrea0\Spotify.exe".to_string()
            })
        });
        assert_eq!(closed, vec![(20, spotify.to_string())]);
    }

    #[test]
    fn protected_and_own_processes_are_never_closed() {
        let p = plan(vec![rule("explorer.exe", Mode::Lock)], Some(Session::start(0, 600, "")));
        let procs = [Running { pid: 30, key: "explorer.exe" }, Running { pid: std::process::id(), key: "explorer.exe" }];
        assert!(to_close(&p, 5, &procs, |_| None).is_empty());
    }

    #[test]
    fn warns_when_switching_in_not_while_staying() {
        let p = plan(vec![rule("mobalytics.exe", Mode::Warn)], Some(Session::start(0, 1500, "")));
        let mob = Some(("mobalytics.exe", None));
        let mut w = WarnTracker::default();
        assert_eq!(w.check(&p, 10, mob), Some("mobalytics.exe".into()), "switched in");
        assert_eq!(w.check(&p, 11, mob), None, "still in front");
        assert_eq!(w.check(&p, 100, mob), None, "staying does not repeat");
    }

    #[test]
    fn switching_back_warns_again_after_a_minute() {
        let p = plan(vec![rule("mobalytics.exe", Mode::Warn)], Some(Session::start(0, 1500, "")));
        let mob = Some(("mobalytics.exe", None));
        let other = Some(("code.exe", None));
        let mut w = WarnTracker::default();
        assert!(w.check(&p, 10, mob).is_some());
        w.check(&p, 20, other);
        assert_eq!(w.check(&p, 30, mob), None, "back within a minute");
        w.check(&p, 40, other);
        assert!(w.check(&p, 75, mob).is_some(), "back after a minute");
    }

    #[test]
    fn no_warning_when_paused_unlocked_or_locked() {
        let mob = Some(("mobalytics.exe", None));
        let mut paused = Session::start(0, 1500, "");
        paused.pause(5);
        let mut w = WarnTracker::default();
        assert_eq!(w.check(&plan(vec![rule("mobalytics.exe", Mode::Warn)], Some(paused)), 10, mob), None);

        let mut unlocked = rule("mobalytics.exe", Mode::Warn);
        unlocked.allowed_until = Some(100);
        assert_eq!(w.check(&plan(vec![unlocked], Some(Session::start(0, 1500, ""))), 10, mob), None);

        let locked = plan(vec![rule("mobalytics.exe", Mode::Lock)], Some(Session::start(0, 1500, "")));
        assert_eq!(w.check(&locked, 10, mob), None, "lock closes it instead");
    }
}
