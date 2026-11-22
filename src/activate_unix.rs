//! Turns fonts on and off on macOS and Linux. On: a symlink to the file in FontCraft's own folder
//! inside the user's fonts folder (the *links folder*, `activate::key()`). Off: that link removed,
//! or, on Linux, a fontconfig reject rule for a font the user put in ~/.local/share/fonts.
//! Only links are ever removed: no font file is ever deleted or moved.
//! Compiled on every OS so its file logic is tested on Windows too; only making a link needs Unix.
#![cfg_attr(not(unix), allow(dead_code))]
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use crate::activate::{link_name, norm};

const SYSTEM_LOCK: &str = "Installed for every user; FontCraft never changes these";
const MAC_LOCK: &str = "macOS manages fonts in ~/Library/Fonts; use Font Book to turn these off";
const LINK_LOCK: &str = "FontCraft's own link; turn off the original font instead";
const MAC: bool = cfg!(target_os = "macos");
/// fontconfig reads `<glob>` as a pattern, so a path holding one of these could hide other fonts.
const GLOB_CHARS: [char; 3] = ['*', '?', '['];
const CONF_HEAD: &str = r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
<!-- Written by FontCraft: fonts you turned off. -->
<selectfont><rejectfont>
"#;
const CONF_TAIL: &str = "</rejectfont></selectfont>\n</fontconfig>\n";

/// Why this file can't be turned on or off here, if it can't.
pub fn lock_reason_in(path: &Path, user_fonts: &Path, links: &Path) -> Option<&'static str> {
    if !links.as_os_str().is_empty() && path.starts_with(links) {
        Some(LINK_LOCK)
    } else if crate::platform::system_font_dirs().iter().any(|d| path.starts_with(d)) {
        Some(SYSTEM_LOCK)
    } else if MAC && is_user_font(path, user_fonts, links) {
        Some(MAC_LOCK)
    } else {
        None
    }
}

/// Put straight into the user's fonts folder (not one of FontCraft's links).
fn is_user_font(path: &Path, user_fonts: &Path, links: &Path) -> bool {
    !user_fonts.as_os_str().is_empty() && path.starts_with(user_fonts) && !path.starts_with(links)
}

fn is_link(p: &Path) -> bool {
    std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink())
}

/// Files the links in the links folder point to, skipping links whose file is gone.
pub fn read_active(links: &str) -> HashSet<PathBuf> {
    let links = Path::new(links);
    std::fs::read_dir(links)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_symlink()))
        .filter_map(|e| std::fs::read_link(e.path()).ok())
        .map(|t| links.join(t))
        .filter(|t| t.is_file())
        .map(|t| norm(&t))
        .collect()
}

/// Fonts turned off by a rule in FontCraft's fontconfig file (never any on macOS).
pub fn read_disabled(conf: &Path) -> HashSet<PathBuf> {
    if MAC {
        return HashSet::new();
    }
    std::fs::read_to_string(conf).map(|t| parse_reject_conf(&t)).unwrap_or_default().iter().map(|p| norm(p)).collect()
}

pub fn turn_on_in(links: &str, conf: &Path, paths: &[PathBuf]) -> Result<usize, String> {
    let n = turn_on_with(&user_fonts(), Path::new(links), conf, paths)?;
    refresh(Path::new(links), n);
    Ok(n)
}

pub fn turn_off_in(links: &str, conf: &Path, paths: &[PathBuf]) -> Result<usize, String> {
    let n = turn_off_with(&user_fonts(), Path::new(links), conf, paths)?;
    refresh(Path::new(links), n);
    Ok(n)
}

fn user_fonts() -> PathBuf {
    crate::platform::user_fonts_dir().unwrap_or_default()
}

/// Refuses the whole batch, before anything changes, if the links folder is unknown or a file is locked.
fn check(user_fonts: &Path, links: &Path, paths: &[PathBuf]) -> Result<(), String> {
    if !links.is_absolute() {
        return Err("can't find your fonts folder (is HOME set?)".into());
    }
    match paths.iter().find_map(|p| lock_reason_in(p, user_fonts, links).map(|r| (p, r))) {
        Some((p, reason)) => Err(format!("{}: {reason}", p.display())),
        None => Ok(()),
    }
}

/// Turns files on: a link for each (an existing correct link counts as done), or on Linux the
/// file's rule removed for a font in the user's fonts folder. Returns how many changed.
fn turn_on_with(user_fonts: &Path, links: &Path, conf: &Path, paths: &[PathBuf]) -> Result<usize, String> {
    check(user_fonts, links, paths)?;
    let (unreject, to_link): (Vec<&PathBuf>, Vec<&PathBuf>) = paths.iter().partition(|p| is_user_font(p, user_fonts, links));
    if let Some(missing) = to_link.iter().find(|p| !p.is_file()) {
        return Err(format!("{}: file not found", missing.display()));
    }
    let mut changed = if unreject.is_empty() { 0 } else { edit_conf(conf, |rules| unreject.iter().filter(|p| rules.remove(**p)).count())? };
    if !to_link.is_empty() {
        std::fs::create_dir_all(links).map_err(|e| format!("can't create {}: {e}", links.display()))?;
    }
    for path in to_link {
        let at = links.join(link_name(path));
        if std::fs::read_link(&at).is_ok_and(|t| t == *path) {
            continue;
        }
        if is_link(&at) {
            // A stale link of ours (its file moved): replace it. Only ever a link, never a file.
            std::fs::remove_file(&at).map_err(|e| format!("can't replace {}: {e}", at.display()))?;
        }
        make_link(path, &at).map_err(|e| format!("can't turn on {}: {e}", path.display()))?;
        changed += 1;
    }
    Ok(changed)
}

/// Turns files off: removes FontCraft's link (even a dangling one; never the file itself), or on
/// Linux adds a rule for a font in the user's fonts folder. Returns how many changed.
fn turn_off_with(user_fonts: &Path, links: &Path, conf: &Path, paths: &[PathBuf]) -> Result<usize, String> {
    check(user_fonts, links, paths)?;
    let mut unlink = Vec::new();
    let mut reject = Vec::new();
    for path in paths {
        let link = links.join(link_name(path));
        if is_link(&link) {
            unlink.push(link);
        } else if is_user_font(path, user_fonts, links) {
            check_rule_path(path)?;
            reject.push(path.clone());
        } else {
            return Err(format!("{}: FontCraft didn't turn this font on, so it can't be turned off here", path.display()));
        }
    }
    let rejected = if reject.is_empty() { 0 } else { edit_conf(conf, |rules| reject.into_iter().filter(|p| rules.insert(p.clone())).count())? };
    for link in &unlink {
        std::fs::remove_file(link).map_err(|e| format!("can't remove the link {}: {e}", link.display()))?;
    }
    Ok(rejected + unlink.len())
}

/// A path fontconfig would read back as exactly this file.
fn check_rule_path(path: &Path) -> Result<(), String> {
    let text = path.to_str().ok_or_else(|| format!("{}: its name isn't valid UTF-8, so it can't be turned off", path.display()))?;
    if let Some(c) = text.chars().find(|c| (*c as u32) < 0x20) {
        return Err(format!("{}: can't be turned off because its path contains a control character (U+{:04X}); rename it first", path.display(), c as u32));
    }
    match text.chars().find(|c| GLOB_CHARS.contains(c)) {
        Some(c) => Err(format!("{}: can't be turned off because its path contains '{c}'; rename it first", path.display())),
        None => Ok(()),
    }
}

/// Applies `change` to the rules in `conf` and saves them when it changed any; returns that count.
fn edit_conf(conf: &Path, change: impl FnOnce(&mut BTreeSet<PathBuf>) -> usize) -> Result<usize, String> {
    let text = match std::fs::read_to_string(conf) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        // Never overwrite rules we couldn't read.
        Err(e) => return Err(format!("can't read {}: {e}", conf.display())),
    };
    let mut rules = parse_reject_conf(&text);
    let n = change(&mut rules);
    if n > 0 {
        write_reject_conf(conf, &rules)?;
    }
    Ok(n)
}

#[cfg(unix)]
fn make_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(not(unix))]
fn make_link(_target: &Path, _link: &Path) -> std::io::Result<()> {
    Err(std::io::Error::other("links are made on macOS and Linux only"))
}

/// Linux: asks fontconfig to rescan now. Best effort: it also notices on its own, and a missing
/// fc-cache is fine. shortcut: blocks the caller while fc-cache runs; move to a thread if it's slow.
fn refresh(links: &Path, changed: usize) {
    if MAC || changed == 0 {
        return;
    }
    let dir = links.parent().unwrap_or(links);
    let _ = std::process::Command::new("fc-cache")
        .arg("-f")
        .arg(dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Saves the rules as FontCraft's fontconfig file: a temp file renamed over it, so a crash never
/// leaves half a file. Creates its folder.
fn write_reject_conf(conf: &Path, paths: &BTreeSet<PathBuf>) -> Result<(), String> {
    let globs: String = paths.iter().map(|p| format!("<glob>{}</glob>\n", escape(&p.to_string_lossy()))).collect();
    let fail = |e: std::io::Error| format!("can't save the list of fonts you turned off ({}): {e}", conf.display());
    if !conf.is_absolute() {
        // A relative path (e.g. from a relative HOME) would write into the current folder.
        return Err(fail(std::io::Error::other("can't find your settings folder (is HOME set?)")));
    }
    if let Some(dir) = conf.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    let tmp = conf.with_extension("conf.tmp");
    std::fs::write(&tmp, format!("{CONF_HEAD}{globs}{CONF_TAIL}")).map_err(fail)?;
    std::fs::rename(&tmp, conf).map_err(fail)
}

/// Exact inverse of `write_reject_conf`.
fn parse_reject_conf(text: &str) -> BTreeSet<PathBuf> {
    text.split("<glob>").skip(1).filter_map(|s| s.split_once("</glob>")).map(|(g, _)| PathBuf::from(unescape(g))).collect()
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temp folder holding a user fonts folder, its links folder and a conf file; removed on drop.
    struct Tmp(PathBuf);

    impl Tmp {
        fn new(name: &str) -> Self {
            let d = std::env::temp_dir().join(format!("fontcraft-unix-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).unwrap();
            Self(d)
        }
        fn fonts(&self) -> PathBuf {
            self.0.join("fonts")
        }
        fn links(&self) -> PathBuf {
            self.fonts().join("FontCraft")
        }
        fn conf(&self) -> PathBuf {
            self.0.join("conf.d").join("99-fontcraft.conf")
        }
        /// A file standing in for a font, in `folder` under the temp folder.
        fn file(&self, folder: &str, name: &str) -> PathBuf {
            let d = self.0.join(folder);
            std::fs::create_dir_all(&d).unwrap();
            let f = d.join(name);
            std::fs::write(&f, b"font").unwrap();
            f
        }
        fn on(&self, paths: &[PathBuf]) -> Result<usize, String> {
            turn_on_with(&self.fonts(), &self.links(), &self.conf(), paths)
        }
        fn off(&self, paths: &[PathBuf]) -> Result<usize, String> {
            turn_off_with(&self.fonts(), &self.links(), &self.conf(), paths)
        }
        #[cfg(unix)]
        fn active(&self) -> HashSet<PathBuf> {
            read_active(self.links().to_str().unwrap())
        }
        #[cfg(unix)]
        fn link_of(&self, f: &Path) -> PathBuf {
            self.links().join(link_name(f))
        }
    }

    impl Drop for Tmp {
        fn drop(&mut self) {
            // Removes links, never what they point to.
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[cfg(unix)]
    #[test]
    fn turn_on_creates_link_and_read_active_sees_target() {
        let t = Tmp::new("on");
        let f = t.file("added", "a.ttf");
        assert_eq!(t.on(&[f.clone()]), Ok(1));
        assert!(is_link(&t.link_of(&f)));
        assert!(t.active().contains(&norm(&f)));
    }

    #[cfg(unix)]
    #[test]
    fn turn_on_twice_is_idempotent() {
        let t = Tmp::new("twice");
        let f = t.file("added", "b.ttf");
        assert_eq!(t.on(&[f.clone()]), Ok(1));
        assert_eq!(t.on(&[f]), Ok(0));
        assert_eq!(std::fs::read_dir(t.links()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn turn_off_removes_link_not_target() {
        let t = Tmp::new("off");
        let f = t.file("added", "c.ttf");
        t.on(&[f.clone()]).unwrap();
        assert_eq!(t.off(&[f.clone()]), Ok(1));
        assert!(f.is_file(), "the font file itself must stay");
        assert!(!is_link(&t.link_of(&f)));
        assert!(t.active().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn turn_off_dangling_link_succeeds() {
        let t = Tmp::new("dangling");
        let f = t.file("added", "d.ttf");
        t.on(&[f.clone()]).unwrap();
        std::fs::remove_file(&f).unwrap();
        assert!(t.active().is_empty(), "dangling links are not on");
        assert_eq!(t.off(&[f.clone()]), Ok(1));
        assert!(!is_link(&t.link_of(&f)));
    }

    #[cfg(unix)]
    #[test]
    fn missing_links_folder_is_created() {
        let t = Tmp::new("mkdir");
        let f = t.file("added", "e.ttf");
        assert!(!t.links().exists());
        assert_eq!(t.on(&[f]), Ok(1));
        assert!(t.links().is_dir());
    }

    #[test]
    fn turn_on_missing_file_errors_and_creates_nothing() {
        let t = Tmp::new("nofile");
        let f = t.0.join("added").join("nope.ttf");
        assert!(t.on(&[f]).unwrap_err().contains("nope.ttf"));
        assert!(!t.links().exists());
    }

    #[test]
    fn turn_off_font_fontcraft_did_not_turn_on_errors() {
        let t = Tmp::new("notours");
        let f = t.file("added", "f.ttf");
        assert!(t.off(&[f]).unwrap_err().contains("can't be turned off here"));
        assert!(!t.conf().exists());
    }

    #[test]
    fn reject_conf_round_trip_with_special_chars() {
        let t = Tmp::new("conf");
        let paths: BTreeSet<PathBuf> = ["/f/A & B <1>.ttf", "/f/خط.ttf"].into_iter().map(PathBuf::from).collect();
        write_reject_conf(&t.conf(), &paths).unwrap();
        let text = std::fs::read_to_string(t.conf()).unwrap();
        assert!(text.contains("<glob>/f/A &amp; B &lt;1&gt;.ttf</glob>"), "{text}");
        assert!(text.starts_with("<?xml version=\"1.0\"?>\n<!DOCTYPE fontconfig SYSTEM \"urn:fontconfig:fonts.dtd\">\n"));
        assert_eq!(parse_reject_conf(&text), paths);
        assert!(!t.0.join("conf.d").join("99-fontcraft.conf.tmp").exists(), "temp file renamed away");
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn turn_off_refuses_control_characters() {
        let t = Tmp::new("ctrl");
        let e = t.off(&[t.fonts().join("ab.ttf")]).unwrap_err();
        assert!(e.contains("control character"), "{e}");
        assert!(!t.conf().exists(), "conf untouched");
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn turn_off_refuses_glob_characters() {
        let t = Tmp::new("glob");
        let other: BTreeSet<PathBuf> = [t.fonts().join("other.ttf")].into_iter().collect();
        write_reject_conf(&t.conf(), &other).unwrap();
        let before = std::fs::read(t.conf()).unwrap();
        let e = t.off(&[t.fonts().join("a[1].ttf")]).unwrap_err();
        assert!(e.contains('['), "{e}");
        assert_eq!(std::fs::read(t.conf()).unwrap(), before, "conf untouched");
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn turn_off_user_font_adds_rule_and_turn_on_removes_it() {
        let t = Tmp::new("rule");
        let f = t.file("fonts", "u.ttf");
        assert_eq!(t.off(&[f.clone()]), Ok(1));
        assert!(read_disabled(&t.conf()).contains(&norm(&f)));
        assert_eq!(t.off(&[f.clone()]), Ok(0), "already off");
        assert_eq!(t.on(&[f.clone()]), Ok(1));
        assert!(read_disabled(&t.conf()).is_empty());
        assert!(f.is_file());
    }

    #[test]
    fn relative_conf_path_is_refused_and_nothing_written() {
        let t = Tmp::new("relconf");
        let rel = PathBuf::from(format!("fontcraft-relconf-{}", std::process::id()));
        let conf = rel.join("99-fontcraft.conf");
        let paths: BTreeSet<PathBuf> = [t.fonts().join("x.ttf")].into_iter().collect();
        assert!(write_reject_conf(&conf, &paths).is_err());
        let f = t.file("fonts", "u.ttf");
        assert!(turn_off_with(&t.fonts(), &t.links(), &conf, &[f]).is_err());
        assert!(!rel.exists(), "nothing created in the current folder");
    }

    #[test]
    fn links_folder_entries_are_locked() {
        let t = Tmp::new("linklock");
        let link = t.links().join("a-0123abcd.ttf");
        let reason = lock_reason_in(&link, &t.fonts(), &t.links()).unwrap();
        assert!(reason.contains("FontCraft's own link"), "{reason}");
        assert!(t.on(&[link.clone()]).unwrap_err().contains("FontCraft's own link"));
        assert!(t.off(&[link]).unwrap_err().contains("FontCraft's own link"));
        assert!(!t.links().exists() && !t.conf().exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn mac_user_font_is_locked_and_conf_not_written() {
        let t = Tmp::new("maclock");
        let f = t.file("fonts", "u.ttf");
        assert!(t.off(&[f.clone()]).unwrap_err().contains("Font Book"));
        assert!(t.on(&[f]).unwrap_err().contains("Font Book"));
        assert!(!t.conf().exists());
    }
}
