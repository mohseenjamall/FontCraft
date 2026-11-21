//! Finds out whether a newer FontCraft release is out. GitHub redirects `releases/latest` to
//! `releases/tag/<tag>`, so the redirect alone names the latest version: no API, JSON or token.
//! Runs once per start on a background thread; offline, blocked or private repo = no update.
use std::process::Command;
use std::sync::mpsc::{Receiver, channel};

use eframe::egui;

pub const LATEST_URL: &str = "https://github.com/mohseenjamall/FontCraft/releases/latest";
const RELEASE_PAGES: &str = "https://github.com/mohseenjamall/FontCraft/releases/";

#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    /// The newer version, without a leading "v".
    pub version: String,
    /// Its release page.
    pub url: String,
}

/// "v1.2.3", "1.2", "1.2.3-beta" → (1, 2, 3), (1, 2, 0), (1, 2, 3). Anything else → None.
fn version(s: &str) -> Option<(u32, u32, u32)> {
    let core = s.trim().trim_start_matches(['v', 'V']).split(['-', '+']).next()?;
    let mut parts = core.split('.').map(str::parse::<u32>);
    let major = parts.next()?.ok()?;
    let minor = parts.next().unwrap_or(Ok(0)).ok()?;
    let patch = parts.next().unwrap_or(Ok(0)).ok()?;
    parts.next().is_none().then_some((major, minor, patch))
}

/// True only when `latest` is a valid version strictly above `current`.
pub fn is_newer(latest: &str, current: &str) -> bool {
    matches!((version(latest), version(current)), (Some(l), Some(c)) if l > c)
}

/// Only FontCraft's own release pages may be opened from the banner.
fn trusted(url: &str) -> bool {
    url.starts_with(RELEASE_PAGES) && !url.contains(['"', ' ', '\r', '\n', '\\']) && !url.contains("..")
}

/// curl's `-w "%{http_code} %{redirect_url}"` output → the trusted release page it redirects to.
fn redirect_target(out: &str) -> Option<&str> {
    let (code, url) = out.trim().split_once(' ')?;
    (code.starts_with('3') && trusted(url)).then_some(url)
}

fn update_for(tag: &str, url: &str, current: &str) -> Option<Update> {
    (is_newer(tag, current) && trusted(url)).then(|| Update { version: tag.trim_start_matches(['v', 'V']).to_owned(), url: url.to_owned() })
}

fn curl() -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut c = Command::new("curl.exe");
        c.creation_flags(CREATE_NO_WINDOW);
        c
    }
    #[cfg(not(windows))]
    Command::new("curl")
}

/// What GitHub answered for `releases/latest`, as curl prints it ("302 https://…" or "404 ").
/// shortcut: uses the curl.exe built into Windows 10 1803+; switch to an HTTP crate if older Windows matters.
fn fetch() -> Option<String> {
    let out = curl().args(["-s", "-o", "NUL", "--max-time", "10", "-w", "%{http_code} %{redirect_url}", LATEST_URL]).output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The newer release, if any. `FONTCRAFT_LATEST=<tag>` stands in for GitHub's answer.
pub fn check(current: &str) -> Option<Update> {
    if let Ok(tag) = std::env::var("FONTCRAFT_LATEST") {
        return update_for(&tag, &format!("{RELEASE_PAGES}tag/{}", tag.trim()), current);
    }
    let answer = fetch()?;
    let url = redirect_target(&answer)?;
    update_for(url.rsplit('/').next()?, url, current)
}

/// Checks in the background; the window is repainted when an update is found.
pub fn spawn_check(ctx: egui::Context) -> Receiver<Update> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        if let Some(u) = check(env!("CARGO_PKG_VERSION")) {
            let _ = tx.send(u);
            ctx.request_repaint();
        }
    });
    rx
}

/// Opens a release page in the default browser.
pub fn open(url: &str) -> Result<(), String> {
    if !trusted(url) {
        return Err(format!("refused to open {url}"));
    }
    let opener = if cfg!(windows) { "explorer" } else { "xdg-open" };
    Command::new(opener).arg(url).spawn().map(drop).map_err(|e| format!("couldn't open the download page: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions() {
        assert_eq!(version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(version("1.2"), Some((1, 2, 0)));
        assert_eq!(version("V2"), Some((2, 0, 0)));
        assert_eq!(version("1.2.3-beta.1"), Some((1, 2, 3)));
        assert_eq!(version("1.2.3.4"), None);
        assert_eq!(version("latest"), None);
        assert_eq!(version(""), None);
    }

    #[test]
    fn newer_only_when_strictly_greater() {
        assert!(is_newer("v1.0.1", "1.0.0"));
        assert!(is_newer("v1.10.0", "1.9.9"));
        assert!(is_newer("2.0", "1.99.99"));
        assert!(!is_newer("v1.0.0", "1.0.0"));
        assert!(!is_newer("v0.9.0", "1.0.0"));
        assert!(!is_newer("nightly", "1.0.0"));
    }

    #[test]
    fn reads_the_redirect_and_ignores_anything_else() {
        let ok = "302 https://github.com/mohseenjamall/FontCraft/releases/tag/v1.1.0";
        assert_eq!(redirect_target(ok), Some("https://github.com/mohseenjamall/FontCraft/releases/tag/v1.1.0"));
        assert_eq!(redirect_target("404 "), None);
        assert_eq!(redirect_target("000 "), None);
        assert_eq!(redirect_target(""), None);
        assert_eq!(redirect_target("302 https://evil.example/releases/tag/v9.0.0"), None);
    }

    #[test]
    fn only_fontcraft_release_pages_are_trusted() {
        assert!(trusted("https://github.com/mohseenjamall/FontCraft/releases/tag/v1.1.0"));
        assert!(!trusted("https://github.com/someone/else/releases/tag/v1.1.0"));
        assert!(!trusted("https://github.com/mohseenjamall/FontCraft/releases/../../evil"));
        assert!(!trusted("https://github.com/mohseenjamall/FontCraft/releases/tag/v1 & calc"));
        assert!(open("file:///C:/Windows/System32/calc.exe").is_err());
    }

    #[test]
    fn update_only_for_a_newer_trusted_release() {
        let url = "https://github.com/mohseenjamall/FontCraft/releases/tag/v1.1.0";
        assert_eq!(update_for("v1.1.0", url, "1.0.0"), Some(Update { version: "1.1.0".into(), url: url.into() }));
        assert_eq!(update_for("v1.0.0", url, "1.0.0"), None);
        assert_eq!(update_for("v1.1.0", "https://evil.example/x", "1.0.0"), None);
    }

    #[test]
    fn override_stands_in_for_github() {
        // Only this test touches FONTCRAFT_LATEST, so it can't race another test.
        unsafe { std::env::set_var("FONTCRAFT_LATEST", "v9.9.9") };
        let newer = check("1.0.0");
        unsafe { std::env::set_var("FONTCRAFT_LATEST", "v1.0.0") };
        let same = check("1.0.0");
        unsafe { std::env::remove_var("FONTCRAFT_LATEST") };
        assert_eq!(newer.map(|u| u.version), Some("9.9.9".into()));
        assert_eq!(same, None);
    }

    /// Asks the real GitHub; run with `--ignored`. Private repo → "404", public → "302 …".
    #[test]
    #[ignore]
    fn live_update_check() {
        let answer = fetch();
        let found = check(env!("CARGO_PKG_VERSION"));
        println!("live update check done: GitHub answered {answer:?}, update = {found:?}");
    }
}
