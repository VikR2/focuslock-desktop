//! Enforces rules while a session runs. Lives on its own thread, so blocking
//! keeps working whatever the window is doing, and stops on its own when the
//! session ends.

use super::session::{now_secs, Session};
use super::store::{Mode, Rule};
use super::{is_protected_exe, process_matches};
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What the blocker needs to know. The UI replaces it on every change.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub rules: Vec<Rule>,
    pub session: Option<Session>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Closed { name: String },
    Warned { name: String },
    /// Windows refused to close it (usually an app running as administrator).
    CouldNotClose { name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Kill { pid: u32, name: String },
    Warn { pid: u32, name: String },
}

/// A running process as the blocker sees it.
pub struct Running<'a> {
    pub pid: u32,
    pub key: &'a str,
}

/// Decide what to do with the running processes. `path_of` is only called for
/// Store-app rules, which match by package folder.
pub fn decide(
    plan: &Plan,
    now: u64,
    procs: &[Running<'_>],
    warned: &HashSet<u32>,
    mut path_of: impl FnMut(u32) -> Option<String>,
) -> Vec<Action> {
    let enforcing = plan.session.as_ref().is_some_and(|s| s.is_enforcing(now));
    if !enforcing {
        return Vec::new();
    }
    let own = std::process::id();
    let active: Vec<&Rule> = plan.rules.iter().filter(|r| !r.is_allowed(now)).collect();
    let mut out = Vec::new();
    for p in procs {
        if p.pid == own || is_protected_exe(p.key) {
            continue;
        }
        let mut path: Option<Option<String>> = None;
        for r in &active {
            let needs_path = r.key.starts_with("pkg:");
            let proc_path = if needs_path {
                path.get_or_insert_with(|| path_of(p.pid)).as_deref()
            } else {
                None
            };
            if !process_matches(&r.key, p.key, proc_path) {
                continue;
            }
            match r.mode {
                Mode::Lock => out.push(Action::Kill { pid: p.pid, name: r.name.clone() }),
                Mode::Warn if !warned.contains(&p.pid) => out.push(Action::Warn { pid: p.pid, name: r.name.clone() }),
                Mode::Warn => {}
            }
            break;
        }
    }
    out
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
    use super::win::procs;
    let mut warned: HashSet<u32> = HashSet::new();
    loop {
        let snapshot = plan.lock().map(|p| p.clone()).unwrap_or_default();
        let now = now_secs();
        if snapshot.session.as_ref().is_some_and(|s| s.is_enforcing(now)) {
            let list = procs::list();
            let running: Vec<Running> = list.iter().map(|p| Running { pid: p.pid, key: &p.key }).collect();
            for action in decide(&snapshot, now, &running, &warned, procs::image_path) {
                let event = match action {
                    Action::Kill { pid, name } => {
                        if procs::kill(pid) {
                            Event::Closed { name }
                        } else {
                            Event::CouldNotClose { name }
                        }
                    }
                    Action::Warn { pid, name } => {
                        warned.insert(pid);
                        Event::Warned { name }
                    }
                };
                if tx.send(event).is_err() {
                    return; // the window is gone
                }
            }
            // Forget warnings for processes that have exited.
            let alive: HashSet<u32> = list.iter().map(|p| p.pid).collect();
            warned.retain(|pid| alive.contains(pid));
        } else {
            warned.clear();
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
        Plan { rules, session }
    }

    #[test]
    fn nothing_happens_without_a_running_session() {
        let p = plan(vec![rule("discord.exe", Mode::Lock)], None);
        let procs = [Running { pid: 10, key: "discord.exe" }];
        assert!(decide(&p, 100, &procs, &HashSet::new(), |_| None).is_empty());

        let mut paused = Session::start(0, 600, "");
        paused.pause(10);
        let p = plan(vec![rule("discord.exe", Mode::Lock)], Some(paused));
        assert!(decide(&p, 100, &procs, &HashSet::new(), |_| None).is_empty());

        let p = plan(vec![rule("discord.exe", Mode::Lock)], Some(Session::start(0, 60, "")));
        assert!(decide(&p, 61, &procs, &HashSet::new(), |_| None).is_empty(), "finished session");
    }

    #[test]
    fn lock_kills_and_warn_warns_once() {
        let p = plan(
            vec![rule("discord.exe", Mode::Lock), rule("steam.exe", Mode::Warn)],
            Some(Session::start(0, 600, "")),
        );
        let procs = [
            Running { pid: 10, key: "discord.exe" },
            Running { pid: 11, key: "steam.exe" },
            Running { pid: 12, key: "code.exe" },
        ];
        let acts = decide(&p, 5, &procs, &HashSet::new(), |_| None);
        assert_eq!(
            acts,
            vec![
                Action::Kill { pid: 10, name: "discord.exe".into() },
                Action::Warn { pid: 11, name: "steam.exe".into() },
            ]
        );
        let warned: HashSet<u32> = [11].into();
        assert_eq!(decide(&p, 6, &procs, &warned, |_| None).len(), 1);
    }

    #[test]
    fn store_rules_match_by_package_folder_and_unlocked_rules_are_skipped() {
        let spotify = "pkg:spotifyab.spotifymusic_zpdnekdrzrea0";
        let mut unlocked = rule("discord.exe", Mode::Lock);
        unlocked.allowed_until = Some(1_000);
        let p = plan(vec![rule(spotify, Mode::Lock), unlocked], Some(Session::start(0, 600, "")));
        let procs = [Running { pid: 20, key: "spotify.exe" }, Running { pid: 21, key: "discord.exe" }];
        let acts = decide(&p, 5, &procs, &HashSet::new(), |pid| {
            (pid == 20).then(|| {
                r"C:\Program Files\WindowsApps\SpotifyAB.SpotifyMusic_1.280.0.0_x64__zpdnekdrzrea0\Spotify.exe".to_string()
            })
        });
        assert_eq!(acts, vec![Action::Kill { pid: 20, name: spotify.into() }]);
    }

    #[test]
    fn protected_and_own_processes_are_never_killed() {
        let p = plan(vec![rule("explorer.exe", Mode::Lock)], Some(Session::start(0, 600, "")));
        let procs = [Running { pid: 30, key: "explorer.exe" }, Running { pid: std::process::id(), key: "explorer.exe" }];
        assert!(decide(&p, 5, &procs, &HashSet::new(), |_| None).is_empty());
    }
}
