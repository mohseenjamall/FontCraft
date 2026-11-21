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
/// shortcut: uses the curl that ships with Windows 10 1803+, macOS and nearly all Linux; switch to an HTTP crate if it must work without curl.
fn fetch() -> Option<String> {
    let null = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let out = curl().args(["-s", "-o", null, "--max-time", "10", "-w", "%{http_code} %{redirect_url}", LATEST_URL]).output().ok()?;
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

const DOWNLOADS: &str = "https://github.com/mohseenjamall/FontCraft/releases/download/";

/// The program file every release must attach for this OS, e.g. `FontCraft-1.2.0-windows-x64.exe`,
/// `FontCraft-1.2.0-linux-x64`, `FontCraft-1.2.0-macos-arm64` / `-macos-x64`.
fn asset_name(version: &str) -> String {
    let platform = if cfg!(windows) {
        "windows-x64.exe"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") { "macos-arm64" } else { "macos-x64" }
    } else {
        "linux-x64"
    };
    format!("FontCraft-{version}-{platform}")
}

/// A downloaded file isn't executable, and `self_replace` copies from it.
#[cfg(unix)]
fn make_executable(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

/// The SHA-256 that `SHA256SUMS.txt` (lines of `<hex>  <file>`) lists for `file`.
fn expected_hash(sums: &str, file: &str) -> Option<String> {
    sums.lines().find_map(|l| {
        let (hash, name) = l.trim().split_once(char::is_whitespace)?;
        (name.trim() == file && hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())).then(|| hash.to_lowercase())
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn download(url: &str, to: &std::path::Path) -> Result<(), String> {
    let out = curl()
        .args(["-fsSL", "--max-time", "300", "-o"])
        .arg(to)
        .arg(url)
        .output()
        .map_err(|e| format!("couldn't start the download: {e}"))?;
    if out.status.success() { Ok(()) } else { Err(format!("download failed ({url})")) }
}

/// Downloads the release's program, checks it against the release's SHA256SUMS.txt and puts it in
/// place of the running program. The new version runs from the next start (see `restart`).
pub fn install(u: &Update) -> Result<(), String> {
    version(&u.version).ok_or_else(|| format!("bad version {}", u.version))?;
    let dir = std::env::temp_dir().join(format!("fontcraft-update-{}", u.version));
    std::fs::create_dir_all(&dir).map_err(|e| format!("couldn't make a download folder: {e}"))?;
    let base = format!("{DOWNLOADS}v{}/", u.version);
    let (exe, sums) = (dir.join(asset_name(&u.version)), dir.join("SHA256SUMS.txt"));
    download(&format!("{base}SHA256SUMS.txt"), &sums)?;
    let listed = std::fs::read_to_string(&sums).map_err(|e| format!("couldn't read the checksums: {e}"))?;
    let want = expected_hash(&listed, &asset_name(&u.version)).ok_or("the release doesn't list a checksum for the program")?;
    download(&format!("{base}{}", asset_name(&u.version)), &exe)?;
    let bytes = std::fs::read(&exe).map_err(|e| format!("couldn't read the download: {e}"))?;
    if sha256_hex(&bytes) != want {
        return Err("the download doesn't match its checksum; nothing was changed".into());
    }
    #[cfg(unix)]
    make_executable(&exe).map_err(|e| format!("couldn't make the download runnable: {e}"))?;
    self_replace::self_replace(&exe).map_err(|e| format!("couldn't replace FontCraft (is it in a protected folder?): {e}"))
}

/// Installs in the background; the result arrives on the receiver and the window repaints.
pub fn spawn_install(u: Update, ctx: egui::Context) -> Receiver<Result<(), String>> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(install(&u));
        ctx.request_repaint();
    });
    rx
}

/// Starts the (now updated) program again; the caller then closes this window.
pub fn restart() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("couldn't find FontCraft: {e}"))?;
    Command::new(exe).spawn().map(drop).map_err(|e| format!("updated, but couldn't restart: {e}"))
}

/// Opens a release page in the default browser.
pub fn open(url: &str) -> Result<(), String> {
    if !trusted(url) {
        return Err(format!("refused to open {url}"));
    }
    let opener = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
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

    #[test]
    fn finds_the_programs_checksum_only() {
        let h = "d3808ce8710df2084c063da78e6a77c862c35c09bf1f66af6e470fe44428056f";
        let name = asset_name("1.1.0");
        let sums = format!("{}  FontCraft-1.1.0-windows-x64.zip\n{h}  {name}\n", "a".repeat(64));
        assert_eq!(expected_hash(&sums, &name), Some(h.to_owned()));
        assert_eq!(expected_hash(&sums, &asset_name("1.2.0")), None);
        assert_eq!(expected_hash(&format!("short  {name}"), &name), None);
    }

    #[test]
    fn asset_name_matches_this_os() {
        #[cfg(windows)]
        assert_eq!(asset_name("1.2.0"), "FontCraft-1.2.0-windows-x64.exe");
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        assert_eq!(asset_name("1.2.0"), "FontCraft-1.2.0-macos-arm64");
        #[cfg(all(target_os = "macos", not(target_arch = "aarch64")))]
        assert_eq!(asset_name("1.2.0"), "FontCraft-1.2.0-macos-x64");
        #[cfg(not(any(windows, target_os = "macos")))]
        assert_eq!(asset_name("1.2.0"), "FontCraft-1.2.0-linux-x64");
    }

    #[cfg(unix)]
    #[test]
    fn make_executable_sets_the_execute_bits() {
        use std::os::unix::fs::PermissionsExt;
        let f = std::env::temp_dir().join(format!("fontcraft-exec-test-{}", std::process::id()));
        std::fs::write(&f, b"x").unwrap();
        make_executable(&f).unwrap();
        let mode = std::fs::metadata(&f).unwrap().permissions().mode();
        let _ = std::fs::remove_file(&f);
        assert_ne!(mode & 0o111, 0);
    }

    #[test]
    fn sha256_matches_a_known_value() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    /// Downloads the real latest release's program and checks it against its checksum, without
    /// replacing anything; run with `--ignored`.
    #[test]
    #[ignore]
    fn live_download_matches_checksum() {
        let latest = redirect_target(&fetch().unwrap()).unwrap().rsplit('/').next().unwrap().trim_start_matches('v').to_owned();
        let dir = std::env::temp_dir().join(format!("fontcraft-live-download-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let base = format!("{DOWNLOADS}v{latest}/");
        let (exe, sums) = (dir.join(asset_name(&latest)), dir.join("SHA256SUMS.txt"));
        download(&format!("{base}SHA256SUMS.txt"), &sums).unwrap();
        download(&format!("{base}{}", asset_name(&latest)), &exe).unwrap();
        let want = expected_hash(&std::fs::read_to_string(&sums).unwrap(), &asset_name(&latest)).unwrap();
        let got = sha256_hex(&std::fs::read(&exe).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(got, want);
        println!("live download ok: v{latest} {got}");
    }

    /// Asks the real GitHub; run with `--ignored`. Private repo → "404", public → "302 …".
    #[test]
    #[ignore]
    fn live_update_check() {
        let answer = fetch();
        let found = check(env!("CARGO_PKG_VERSION"));
        // What a copy of the first release would see.
        let for_old = check("1.0.0");
        println!("live update check done: GitHub answered {answer:?}, update = {found:?}, for 1.0.0 = {for_old:?}");
    }
}
