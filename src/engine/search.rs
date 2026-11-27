//! In-memory app search. Runs on every keystroke, so it never touches the disk.

use super::catalog::AppEntry;

/// Score how well `query` matches an app. `None` means no match.
pub fn score(query: &str, name: &str, key: &str) -> Option<u32> {
    let q = normalize(query);
    if q.is_empty() {
        return Some(0);
    }
    let name_n = normalize(name);
    let stem = normalize(key.trim_end_matches(".exe"));
    let words: Vec<&str> = name_n.split(' ').filter(|w| !w.is_empty()).collect();
    let compact_q: String = q.chars().filter(|c| *c != ' ').collect();

    let mut best = None::<u32>;
    let mut take = |s: u32| best = Some(best.map_or(s, |b| b.max(s)));

    if name_n == q || stem == compact_q {
        take(1000);
    }
    if name_n.starts_with(&q) {
        take(900);
    }
    if stem.starts_with(&compact_q) {
        take(850);
    }
    if words.iter().any(|w| w.starts_with(&q)) {
        take(800);
    }
    // Initials: "vsc" finds "Visual Studio Code".
    let initials: String = words.iter().filter_map(|w| w.chars().next()).collect();
    if compact_q.len() >= 2 && initials.starts_with(&compact_q) {
        take(750);
    }
    // Every query word starts some name word: "studio vis" finds "Visual Studio".
    let q_words: Vec<&str> = q.split(' ').filter(|w| !w.is_empty()).collect();
    if q_words.len() > 1 && q_words.iter().all(|qw| words.iter().any(|w| w.starts_with(qw))) {
        take(700);
    }
    // Mid-word matches only from 3 letters on, so "sp" doesn't find "Workspace".
    if compact_q.len() >= 3 {
        if name_n.contains(&q) {
            take(600);
        }
        if stem.contains(&compact_q) {
            take(550);
        }
    }
    // Letters in order from the start of a word, for typos of omission:
    // "spotfy" finds "Spotify". Few gaps allowed, so "tra" can't find "Telegram".
    if compact_q.len() >= 4 {
        let first = compact_q.chars().next();
        for (i, w) in words.iter().enumerate() {
            if w.chars().next() != first {
                continue;
            }
            let tail: String = words[i..].concat();
            if let Some(gaps) = subsequence_gaps(&compact_q, &tail) {
                if gaps <= 2 {
                    take(400 - gaps * 50);
                }
            }
        }
    }
    best
}

/// Indices of matching apps, best first. Running apps win ties.
pub fn filter(apps: &[AppEntry], query: &str) -> Vec<usize> {
    let mut hits: Vec<(u32, usize)> = apps
        .iter()
        .enumerate()
        .filter_map(|(i, a)| score(query, &a.name, &a.key).map(|s| (s + a.running as u32 * 5, i)))
        .collect();
    hits.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| apps[a.1].name.to_lowercase().cmp(&apps[b.1].name.to_lowercase()))
    });
    hits.into_iter().map(|(_, i)| i).collect()
}

fn normalize(s: &str) -> String {
    let mapped: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn subsequence_gaps(needle: &str, hay: &str) -> Option<u32> {
    let mut gaps = 0;
    let mut hay_chars = hay.chars();
    for n in needle.chars() {
        loop {
            match hay_chars.next() {
                Some(h) if h == n => break,
                Some(_) => gaps += 1,
                None => return None,
            }
        }
    }
    Some(gaps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::catalog::IconSource;

    fn app(name: &str, key: &str) -> AppEntry {
        AppEntry {
            key: key.into(),
            name: name.into(),
            exe_path: String::new(),
            icon: IconSource::Path(String::new()),
            running: false,
        }
    }

    #[test]
    fn prefix_beats_substring() {
        let apps = vec![app("Microsoft Teams", "ms-teams.exe"), app("TeamViewer", "teamviewer.exe")];
        let order = filter(&apps, "team");
        assert_eq!(apps[order[0]].name, "TeamViewer");
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn initials_and_exe_names_match() {
        assert!(score("vsc", "Visual Studio Code", "code.exe").is_some());
        assert!(score("code", "Visual Studio Code", "code.exe").unwrap() >= 800);
        assert!(score("ninjatrader", "NinjaTrader 8", "ninjatrader.exe").unwrap() >= 850);
    }

    #[test]
    fn case_spacing_and_punctuation_are_ignored() {
        assert!(score("  SPOTIFY ", "Spotify", "spotify.exe").unwrap() >= 1000);
        assert!(score("trading view", "TradingView", "tradingview.exe").unwrap() >= 1000);
    }

    #[test]
    fn missing_letters_still_match_but_rank_low() {
        let s = score("spotfy", "Spotify", "spotify.exe").unwrap();
        assert!((100..600).contains(&s));
        assert!(score("zzz", "Spotify", "spotify.exe").is_none());
        assert!(score("tra", "Telegram", "telegram.exe").is_none());
    }

    #[test]
    fn two_letters_only_match_word_starts() {
        assert!(score("sp", "Spotify", "spotify.exe").is_some());
        assert!(score("sp", "Citrix Workspace", "selfservice.exe").is_none());
        assert!(score("ws", "Citrix Workspace", "selfservice.exe").is_none());
        assert!(score("cw", "Citrix Workspace", "selfservice.exe").is_some());
    }

    #[test]
    fn path_text_does_not_match() {
        // The old search matched "pro" against every path under Program Files.
        assert!(score("program", "Steam", "steam.exe").is_none());
    }
}
