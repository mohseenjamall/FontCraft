//! Turns font files on and off for every Windows app, the way FontBase's own add-on does
//! (registerFonts 0x180001200 / unregisterFonts 0x1800017f0): AddFontResourceW, a value in the
//! current user's font registry key so Windows loads it again after sign-in, and WM_FONTCHANGE.
//! FontCraft writes its own " (FontCraft)" values and never edits FontBase's " (FontBase)" ones.
#[cfg(not(windows))]
use std::collections::HashSet;
use std::path::Path;
#[cfg(any(test, not(windows)))]
use std::path::PathBuf;

pub const FONTS_KEY: &str = r"Software\Microsoft\Windows NT\CurrentVersion\Fonts";
const OWN_SUFFIX: &str = " (FontCraft)";
const FONTBASE_SUFFIX: &str = " (FontBase)";

/// Registry value name for a file: "<stem> <fnv1a-32 of the lowercased path> (FontCraft)".
/// The hash keeps two files with the same name in different folders apart.
pub fn marker(path: &Path) -> String {
    let lower = path.to_string_lossy().to_lowercase();
    let hash = lower.bytes().fold(0x811c_9dc5u32, |h, b| (h ^ b as u32).wrapping_mul(0x0100_0193));
    let stem = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    format!("{stem} {hash:08x}{OWN_SUFFIX}")
}

#[cfg(windows)]
mod imp {
    use std::collections::HashSet;
    use std::ffi::OsString;
    use std::io::ErrorKind;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};

    use windows_sys::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
    use windows_sys::Win32::UI::WindowsAndMessaging::{HWND_BROADCAST, PostMessageW, WM_FONTCHANGE};
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};

    use super::{FONTBASE_SUFFIX, OWN_SUFFIX, marker};

    fn err(what: &str, path: &Path, e: impl std::fmt::Display) -> String {
        format!("{what} {}: {e}", path.display())
    }

    pub fn write_marker(key: &str, path: &Path) -> Result<(), String> {
        let (k, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(key).map_err(|e| err("can't open the font list for", path, e))?;
        k.set_value(marker(path), &path.as_os_str()).map_err(|e| err("can't remember", path, e))
    }

    pub fn remove_marker(key: &str, path: &Path) -> Result<(), String> {
        let Ok(k) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(key, winreg::enums::KEY_SET_VALUE) else {
            return Ok(()); // no key: nothing to remove
        };
        match k.delete_value(marker(path)) {
            Err(e) if e.kind() != ErrorKind::NotFound => Err(err("can't forget", path, e)),
            _ => Ok(()),
        }
    }

    /// Files turned on by FontCraft or FontBase that still exist.
    pub fn read_active(key: &str) -> HashSet<PathBuf> {
        let Ok(k) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(key, KEY_READ) else {
            return HashSet::new();
        };
        k.enum_values()
            .flatten()
            .filter(|(name, _)| name.ends_with(OWN_SUFFIX) || name.contains(FONTBASE_SUFFIX))
            .filter_map(|(name, _)| k.get_value::<OsString, _>(&name).ok())
            .map(PathBuf::from)
            .filter(|p| p.is_file())
            .collect()
    }

    /// True only when FontCraft's own marker for this file exists.
    pub fn is_owned(key: &str, path: &Path) -> bool {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(key, KEY_READ)
            .is_ok_and(|k| k.get_value::<OsString, _>(marker(path)).is_ok())
    }

    fn wide(p: &Path) -> Vec<u16> {
        p.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    fn broadcast() {
        // SAFETY: posting a parameterless broadcast message has no memory requirements.
        unsafe { PostMessageW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0) };
    }

    pub fn activate_in(key: &str, paths: &[PathBuf]) -> Result<usize, String> {
        // Check every file first so a bad one leaves nothing half done.
        if let Some(missing) = paths.iter().find(|p| !p.is_file()) {
            return Err(format!("{}: file not found", missing.display()));
        }
        let mut added = 0;
        let mut result = Ok(());
        for path in paths.iter().filter(|p| !is_owned(key, p)) {
            // SAFETY: `wide` returns a NUL-terminated UTF-16 buffer that outlives the call.
            if unsafe { AddFontResourceW(wide(path).as_ptr()) } == 0 {
                result = Err(format!("{}: Windows couldn't load this font", path.display()));
                break;
            }
            if let Err(e) = write_marker(key, path) {
                // SAFETY: as above; undo the load so state matches the registry.
                unsafe { RemoveFontResourceW(wide(path).as_ptr()) };
                result = Err(e);
                break;
            }
            added += 1;
        }
        if added > 0 {
            broadcast();
        }
        result.map(|()| added)
    }

    pub fn deactivate_in(key: &str, paths: &[PathBuf]) -> Result<usize, String> {
        let mut removed = 0;
        let mut result = Ok(());
        for path in paths.iter().filter(|p| is_owned(key, p)) {
            // SAFETY: NUL-terminated buffer. A 0 return is fine: the file may already be gone.
            unsafe { RemoveFontResourceW(wide(path).as_ptr()) };
            if let Err(e) = remove_marker(key, path) {
                result = Err(e);
                break;
            }
            removed += 1;
        }
        if removed > 0 {
            broadcast();
        }
        result.map(|()| removed)
    }
}

#[cfg(windows)]
pub use imp::{activate_in, deactivate_in, read_active};

#[cfg(not(windows))]
pub fn read_active(_key: &str) -> HashSet<PathBuf> {
    HashSet::new()
}

#[cfg(not(windows))]
pub fn activate_in(_key: &str, _paths: &[PathBuf]) -> Result<usize, String> {
    Err("Turning fonts on and off works on Windows only".into())
}

#[cfg(not(windows))]
pub fn deactivate_in(_key: &str, _paths: &[PathBuf]) -> Result<usize, String> {
    Err("Turning fonts on and off works on Windows only".into())
}

#[cfg(test)]
#[cfg(windows)]
pub(crate) mod tests {
    use super::imp::*;
    use super::*;
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    /// A scratch registry key and temp folder, both removed on drop.
    pub(crate) struct Scratch {
        pub key: String,
        pub dir: PathBuf,
    }

    impl Scratch {
        pub(crate) fn new(name: &str) -> Self {
            let key = format!(r"Software\FontCraftTest\{name}");
            let dir = std::env::temp_dir().join(format!("fontcraft-test-{name}-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            Self { key, dir }
        }

        /// A real font file (a copy of Arial) under a given file name.
        pub(crate) fn font(&self, file: &str) -> PathBuf {
            let src = PathBuf::from(std::env::var_os("WINDIR").unwrap()).join(r"Fonts\arial.ttf");
            let dst = self.dir.join(file);
            std::fs::copy(src, &dst).unwrap();
            dst
        }

        fn value_count(&self) -> usize {
            RegKey::predef(HKEY_CURRENT_USER).open_subkey(&self.key).map_or(0, |k| k.enum_values().count())
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let files: Vec<PathBuf> = std::fs::read_dir(&self.dir).into_iter().flatten().flatten().map(|e| e.path()).collect();
            let _ = deactivate_in(&self.key, &files);
            let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.key);
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn marker_is_stable_and_case_insensitive() {
        // Windows paths ignore case, so the hash must too; the stem keeps its case for readability.
        let hash = |m: String| m.split(' ').nth(1).unwrap().to_owned();
        assert_eq!(marker(Path::new(r"C:\A\Foo.ttf")), marker(Path::new(r"C:\A\Foo.ttf")));
        assert_eq!(hash(marker(Path::new(r"C:\A\Foo.ttf"))), hash(marker(Path::new(r"c:\a\foo.ttf"))));
    }

    #[test]
    fn marker_differs_for_same_stem_in_two_folders() {
        assert_ne!(marker(Path::new(r"C:\A\x.ttf")), marker(Path::new(r"C:\B\x.ttf")));
    }

    #[test]
    fn marker_has_expected_shape() {
        let m = marker(Path::new(r"C:\A\Foo.ttf"));
        let hex = m.strip_prefix("Foo ").and_then(|r| r.strip_suffix(" (FontCraft)")).unwrap();
        assert_eq!(hex.len(), 8);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn write_then_read_then_remove_roundtrip() {
        let s = Scratch::new("roundtrip");
        let f = s.font("a.ttf");
        write_marker(&s.key, &f).unwrap();
        assert!(read_active(&s.key).contains(&f));
        remove_marker(&s.key, &f).unwrap();
        assert!(read_active(&s.key).is_empty());
    }

    #[test]
    fn read_active_skips_missing_files() {
        let s = Scratch::new("missing");
        let f = s.font("gone.ttf");
        write_marker(&s.key, &f).unwrap();
        std::fs::remove_file(&f).unwrap();
        assert!(read_active(&s.key).is_empty());
    }

    #[test]
    fn read_active_includes_fontbase_values() {
        let s = Scratch::new("fbread");
        let f = s.font("fb.ttf");
        let (k, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(&s.key).unwrap();
        k.set_value("Bar (FontBase)\n", &f.as_os_str()).unwrap();
        assert!(read_active(&s.key).contains(&f));
    }

    #[test]
    fn is_owned_false_for_fontbase_value() {
        let s = Scratch::new("fbowned");
        let f = s.font("fb.ttf");
        let (k, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(&s.key).unwrap();
        k.set_value("Bar (FontBase)\n", &f.as_os_str()).unwrap();
        assert!(!is_owned(&s.key, &f));
    }

    #[test]
    fn write_marker_twice_is_idempotent() {
        let s = Scratch::new("twice");
        let f = s.font("t.ttf");
        write_marker(&s.key, &f).unwrap();
        write_marker(&s.key, &f).unwrap();
        assert_eq!(s.value_count(), 1);
    }

    #[test]
    fn unicode_path_roundtrip() {
        let s = Scratch::new("unicode");
        let f = s.font("خط-字.ttf");
        write_marker(&s.key, &f).unwrap();
        assert!(read_active(&s.key).contains(&f));
        assert!(is_owned(&s.key, &f));
    }

    #[test]
    fn activate_makes_font_active_and_loads_it() {
        let s = Scratch::new("activate");
        let f = s.font("act.ttf");
        assert_eq!(activate_in(&s.key, &[f.clone()]), Ok(1));
        assert!(read_active(&s.key).contains(&f));
        // Windows counts loads: a second add succeeds only because the file is a valid, loaded font.
        let w: Vec<u16> = { use std::os::windows::ffi::OsStrExt; f.as_os_str().encode_wide().chain(Some(0)).collect() };
        unsafe {
            assert!(windows_sys::Win32::Graphics::Gdi::AddFontResourceW(w.as_ptr()) >= 1);
            windows_sys::Win32::Graphics::Gdi::RemoveFontResourceW(w.as_ptr());
        }
    }

    #[test]
    fn activate_twice_reports_zero_new() {
        let s = Scratch::new("acttwice");
        let f = s.font("t2.ttf");
        assert_eq!(activate_in(&s.key, &[f.clone()]), Ok(1));
        assert_eq!(activate_in(&s.key, &[f]), Ok(0));
        assert_eq!(s.value_count(), 1);
    }

    #[test]
    fn deactivate_removes_marker_and_resource() {
        let s = Scratch::new("deact");
        let f = s.font("d.ttf");
        activate_in(&s.key, &[f.clone()]).unwrap();
        assert_eq!(deactivate_in(&s.key, &[f.clone()]), Ok(1));
        assert!(read_active(&s.key).is_empty());
        assert!(!is_owned(&s.key, &f));
    }

    #[test]
    fn deactivate_missing_file_still_removes_marker() {
        let s = Scratch::new("deactgone");
        let f = s.font("dg.ttf");
        activate_in(&s.key, &[f.clone()]).unwrap();
        // Windows holds the file open while loaded; release it before deleting, as a user removing it would after sign-out.
        let w: Vec<u16> = { use std::os::windows::ffi::OsStrExt; f.as_os_str().encode_wide().chain(Some(0)).collect() };
        unsafe { windows_sys::Win32::Graphics::Gdi::RemoveFontResourceW(w.as_ptr()) };
        std::fs::remove_file(&f).unwrap();
        assert_eq!(deactivate_in(&s.key, &[f.clone()]), Ok(1));
        assert!(!is_owned(&s.key, &f));
    }

    #[test]
    fn deactivate_ignores_fontbase_values() {
        let s = Scratch::new("deactfb");
        let f = s.font("fb2.ttf");
        let (k, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(&s.key).unwrap();
        k.set_value("Bar (FontBase)\n", &f.as_os_str()).unwrap();
        assert_eq!(deactivate_in(&s.key, &[f.clone()]), Ok(0));
        assert!(k.get_value::<std::ffi::OsString, _>("Bar (FontBase)\n").is_ok());
    }

    #[test]
    fn activate_nonexistent_file_errors() {
        let s = Scratch::new("nofile");
        let f = s.dir.join("nope.ttf");
        let e = activate_in(&s.key, &[f.clone()]).unwrap_err();
        assert!(e.contains("nope.ttf"), "{e}");
        assert_eq!(s.value_count(), 0);
    }

    /// Live check on the real per-user font key; run with `--ignored`. Cleans up after itself.
    #[test]
    #[ignore]
    fn live_activation_on_real_key() {
        let s = Scratch::new("live");
        let f = s.font("FontCraftLiveCheck.ttf");
        let reg = || {
            let out = std::process::Command::new("reg").args(["query", &format!(r"HKCU\{}", super::FONTS_KEY)]).output().unwrap();
            String::from_utf8_lossy(&out.stdout).contains("FontCraftLiveCheck")
        };
        assert_eq!(activate_in(super::FONTS_KEY, &[f.clone()]), Ok(1));
        let on = reg();
        let result = deactivate_in(super::FONTS_KEY, &[f]);
        assert!(on, "marker not visible to reg.exe after activate");
        assert_eq!(result, Ok(1));
        assert!(!reg(), "marker still present after deactivate");
        println!("live activation ok");
    }

    #[test]
    fn activate_unicode_path() {
        let s = Scratch::new("actuni");
        let f = s.font("خط عربي.ttf");
        assert_eq!(activate_in(&s.key, &[f.clone()]), Ok(1));
        assert!(read_active(&s.key).contains(&f));
        assert_eq!(deactivate_in(&s.key, &[f]), Ok(1));
    }
}
