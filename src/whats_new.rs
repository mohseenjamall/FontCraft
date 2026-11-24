//! The "What's new" notes: which bullets belong to a version and when the banner shows.
use crate::update::is_newer;

/// All release notes, newest first, one `## {version}` section each.
pub const NOTES: &str = include_str!("../assets/whats-new.md");

/// The bullets of the `## {version}` section; `None` when it is missing or has none.
pub fn parse_notes(markdown: &str, version: &str) -> Option<Vec<String>> {
    let heading = format!("## {version}");
    let bullets: Vec<String> = markdown
        .lines()
        .skip_while(|l| l.trim() != heading)
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with("## "))
        .filter_map(|l| l.trim().strip_prefix("- ").map(|b| b.trim().to_owned()))
        .filter(|b| !b.is_empty())
        .collect();
    (!bullets.is_empty()).then_some(bullets)
}

/// True only when `seen` is a usable version and `current` is newer than it.
pub fn should_show(seen: Option<&str>, current: &str) -> bool {
    seen.is_some_and(|s| is_newer(current, s.trim()))
}

/// What to do at startup: the notes to show (if any) and whether to remember this version.
#[derive(Debug, PartialEq)]
pub struct Start {
    pub notes: Option<Vec<String>>,
    pub save_version: bool,
}

/// A fresh install (or an unreadable seen file) shows nothing and records the version; an update shows
/// the current version's notes once; the same or an older version changes nothing.
pub fn on_start(seen: Option<&str>, current: &str, markdown: &str) -> Start {
    let newer = should_show(seen, current);
    let same_or_older = seen.is_some_and(|s| crate::update::is_newer(s.trim(), current) || s.trim() == current);
    Start { notes: if newer { parse_notes(markdown, current) } else { None }, save_version: !same_or_older }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD: &str = "## 1.4.0\n- Collections\n- Grid view\n\n## 1.3.0\n- Older thing\n";

    #[test]
    fn parses_a_versions_bullets() {
        assert_eq!(parse_notes(MD, "1.4.0"), Some(vec!["Collections".into(), "Grid view".into()]));
        assert_eq!(parse_notes(MD, "1.3.0"), Some(vec!["Older thing".into()]));
        assert_eq!(parse_notes(MD, "9.9.9"), None);
        assert_eq!(parse_notes("## 1.0.0\nno bullets here\n", "1.0.0"), None);
    }

    #[test]
    fn parsing_handles_windows_line_endings_and_indents() {
        assert_eq!(
            parse_notes("## 1.4.0\r\n- One\r\n  - Two  \r\n\r\n## 1.3.0\r\n- Old\r\n", "1.4.0"),
            Some(vec!["One".into(), "Two".into()])
        );
    }

    #[test]
    fn show_rules() {
        assert!(should_show(Some("1.2.0"), "1.3.0"));
        assert!(!should_show(Some("1.3.0"), "1.3.0")); // same version
        assert!(!should_show(Some("1.4.0"), "1.3.0")); // downgrade
        for seen in [None, Some(""), Some("  "), Some("abc")] {
            assert!(!should_show(seen, "1.3.0"), "{seen:?}");
        }
    }

    #[test]
    fn on_start_decides_notes_and_saving() {
        let fresh = on_start(None, "1.4.0", MD);
        assert_eq!((fresh.notes, fresh.save_version), (None, true));
        let upgraded = on_start(Some("1.1.0"), "1.4.0", MD); // skipped versions: only 1.4.0's notes
        assert_eq!(upgraded.notes, Some(vec!["Collections".into(), "Grid view".into()]));
        assert!(upgraded.save_version);
        let no_notes = on_start(Some("1.1.0"), "2.0.0", MD); // new version without notes: nothing shown, still recorded
        assert_eq!((no_notes.notes, no_notes.save_version), (None, true));
        let same = on_start(Some("1.4.0"), "1.4.0", MD);
        assert_eq!((same.notes, same.save_version), (None, false));
        let downgrade = on_start(Some("1.5.0"), "1.4.0", MD);
        assert_eq!((downgrade.notes, downgrade.save_version), (None, false));
        let garbage = on_start(Some("abc"), "1.4.0", MD);
        assert_eq!((garbage.notes, garbage.save_version), (None, true));
    }

    #[test]
    fn bundled_notes_cover_the_current_version() {
        assert!(parse_notes(NOTES, env!("CARGO_PKG_VERSION")).is_some_and(|n| (1..=6).contains(&n.len())));
    }
}
