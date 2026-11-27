//! A focus session, timed from its end time so it never drifts.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub started_at: u64,
    pub length_secs: u64,
    /// When the session ends if it is not paused again.
    pub ends_at: u64,
    /// Set while paused: the time that was left when paused.
    pub paused_left: Option<u64>,
    #[serde(default)]
    pub reason: String,
    /// Focus group id: only that group's apps may stay open.
    #[serde(default)]
    pub group: Option<String>,
    /// Apps outside the group unlocked for a while: (app key, until).
    #[serde(default)]
    pub allowed: Vec<(String, u64)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Running,
    Paused,
    Finished,
}

impl Session {
    pub fn start(now: u64, length_secs: u64, reason: impl Into<String>) -> Self {
        Self {
            started_at: now,
            length_secs,
            ends_at: now + length_secs,
            paused_left: None,
            reason: reason.into(),
            group: None,
            allowed: Vec::new(),
        }
    }

    pub fn is_allowed(&self, key: &str, now: u64) -> bool {
        self.allowed.iter().any(|(k, until)| k == key && now < *until)
    }

    pub fn phase(&self, now: u64) -> Phase {
        if self.paused_left.is_some() {
            Phase::Paused
        } else if now >= self.ends_at {
            Phase::Finished
        } else {
            Phase::Running
        }
    }

    pub fn left(&self, now: u64) -> u64 {
        self.paused_left.unwrap_or_else(|| self.ends_at.saturating_sub(now))
    }

    /// Fraction of the session already done, 0.0 to 1.0.
    pub fn progress(&self, now: u64) -> f32 {
        if self.length_secs == 0 {
            return 1.0;
        }
        1.0 - (self.left(now) as f32 / self.length_secs as f32).clamp(0.0, 1.0)
    }

    /// Apps are blocked only while the session is running.
    pub fn is_enforcing(&self, now: u64) -> bool {
        self.phase(now) == Phase::Running
    }

    pub fn pause(&mut self, now: u64) {
        if self.phase(now) == Phase::Running {
            self.paused_left = Some(self.left(now));
        }
    }

    pub fn resume(&mut self, now: u64) {
        if let Some(left) = self.paused_left.take() {
            self.ends_at = now + left;
        }
    }
}

/// `25:00`, or `1:05:00` past an hour.
pub fn format_clock(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_down_from_end_time() {
        let s = Session::start(1_000, 1_500, "");
        assert_eq!(s.left(1_000), 1_500);
        assert_eq!(s.left(1_600), 900);
        assert_eq!(s.phase(2_500), Phase::Finished);
        assert_eq!(s.left(9_999), 0);
    }

    #[test]
    fn pause_freezes_time_and_stops_blocking() {
        let mut s = Session::start(0, 600, "");
        s.pause(100);
        assert_eq!(s.phase(5_000), Phase::Paused);
        assert!(!s.is_enforcing(5_000));
        assert_eq!(s.left(5_000), 500);
        s.resume(5_000);
        assert!(s.is_enforcing(5_001));
        assert_eq!(s.left(5_100), 400);
    }

    #[test]
    fn finished_session_does_not_block() {
        let s = Session::start(0, 60, "");
        assert!(!s.is_enforcing(61));
    }

    #[test]
    fn clock_format() {
        assert_eq!(format_clock(1500), "25:00");
        assert_eq!(format_clock(3905), "1:05:05");
    }
}
