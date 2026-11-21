//! Turns font files on and off for every Windows app, the way FontBase's own add-on does
//! (registerFonts 0x180001200 / unregisterFonts 0x1800017f0): AddFontResourceW, a value in the
//! current user's font registry key so Windows loads it again after sign-in, and WM_FONTCHANGE.
//! FontCraft writes its own " (FontCraft)" values. Turning off a user-installed or FontBase font
//! saves its values in the saved list first (`disabled.rs`) so turning it on restores them exactly.
//! System fonts (%WINDIR%\Fonts, HKLM) are never touched; no font file is ever deleted.
//! On macOS and Linux the same functions come from `activate_unix.rs` (links and fontconfig rules).
use std::path::{Path, PathBuf};

pub const FONTS_KEY: &str = r"Software\Microsoft\Windows NT\CurrentVersion\Fonts";
#[cfg(windows)]
const OWN_SUFFIX: &str = " (FontCraft)";

/// Where turned-on fonts are recorded: the registry key on Windows, the links folder elsewhere.
#[allow(dead_code)] // app.rs moves from FONTS_KEY to this in the next change
pub fn key() -> String {
    if cfg!(windows) {
        FONTS_KEY.into()
    } else {
        crate::platform::user_fonts_dir().map(|d| d.join("FontCraft").to_string_lossy().into_owned()).unwrap_or_default()
    }
}

/// Path as a set key: Windows treats paths without regard to case and `/` the same as `\`;
/// macOS and Linux paths are compared as they are.
pub fn norm(path: &Path) -> PathBuf {
    if cfg!(windows) { path.to_string_lossy().to_lowercase().replace('/', "\\").into() } else { path.to_path_buf() }
}

/// Why this file can't be turned on or off, or `None` when it can.
#[allow(dead_code)] // app.rs uses this in the next change
pub fn lock_reason(path: &Path) -> Option<&'static str> {
    #[cfg(windows)]
    return is_system_font(path).then_some("Installed for every user in Windows\\Fonts; FontCraft never changes these");
    #[cfg(not(windows))]
    {
        let user_fonts = crate::platform::user_fonts_dir().unwrap_or_default();
        crate::activate_unix::lock_reason_in(path, &user_fonts, Path::new(&key()))
    }
}

/// Link file name for a font: "<stem>-<fnv1a-32 of the path bytes>.<ext>"; the hash keeps two
/// files with the same name in different folders apart.
pub fn link_name(path: &Path) -> String {
    let hash = path.as_os_str().as_encoded_bytes().iter().fold(0x811c_9dc5u32, |h, &b| (h ^ b as u32).wrapping_mul(0x0100_0193));
    let stem = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    match path.extension() {
        Some(ext) => format!("{stem}-{hash:08x}.{}", ext.to_string_lossy()),
        None => format!("{stem}-{hash:08x}"),
    }
}

/// A file directly in %WINDIR%\Fonts: installed for every user through HKLM, so turning it off
/// would need administrator rights and could break Windows. FontCraft never touches these.
pub fn is_system_font(path: &Path) -> bool {
    let Some(windir) = std::env::var_os("WINDIR") else { return false };
    let fonts = norm(&Path::new(&windir).join("Fonts"));
    path.parent().is_some_and(|p| norm(p) == fonts)
}

#[cfg(windows)]
fn system_error(path: &Path) -> String {
    format!("{}: System font: needs administrator", path.display())
}

/// Registry value name for a file: "<stem> <fnv1a-32 of the lowercased path> (FontCraft)".
/// The hash keeps two files with the same name in different folders apart.
#[cfg(windows)]
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
    use winreg::enums::*;
    use winreg::{RegKey, RegValue};

    use super::{OWN_SUFFIX, is_system_font, marker, norm, system_error};
    use crate::disabled::{Entry, Saved, data_path};

    fn err(what: &str, path: &Path, e: impl std::fmt::Display) -> String {
        format!("{what} {}: {e}", path.display())
    }

    pub fn write_marker(key: &str, path: &Path) -> Result<(), String> {
        let (k, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(key).map_err(|e| err("can't open the font list for", path, e))?;
        k.set_value(marker(path), &path.as_os_str()).map_err(|e| err("can't remember", path, e))
    }

    #[cfg(test)]
    pub fn remove_marker(key: &str, path: &Path) -> Result<(), String> {
        let Ok(k) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(key, winreg::enums::KEY_SET_VALUE) else {
            return Ok(()); // no key: nothing to remove
        };
        match k.delete_value(marker(path)) {
            Err(e) if e.kind() != ErrorKind::NotFound => Err(err("can't forget", path, e)),
            _ => Ok(()),
        }
    }

    /// Files on for this user (installed, or turned on by FontCraft or FontBase) that still exist,
    /// as `norm` keys.
    pub fn read_active(key: &str) -> HashSet<PathBuf> {
        let Ok(k) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(key, KEY_READ) else {
            return HashSet::new();
        };
        k.enum_values()
            .flatten()
            .filter_map(|(_, v)| data_path(v.vtype as u32, &v.bytes))
            .filter(|p| p.is_file())
            .map(|p| norm(&p))
            .collect()
    }

    /// Files turned off by FontCraft whose registry values are in the saved list, as `norm` keys.
    pub fn read_disabled(store: &Path) -> HashSet<PathBuf> {
        Saved::load(store).entries.iter().filter_map(|e| e.path()).map(|p| norm(&p)).collect()
    }

    /// Every value in the key whose data is this file, with its raw type and bytes.
    fn values_for(k: &RegKey, path: &Path) -> Vec<(String, RegValue<'static>)> {
        let want = norm(path);
        k.enum_values()
            .flatten()
            .filter(|(_, v)| data_path(v.vtype.clone() as u32, &v.bytes).is_some_and(|p| norm(&p) == want))
            .collect()
    }

    fn reg_type(n: u32) -> Option<RegType> {
        [REG_NONE, REG_SZ, REG_EXPAND_SZ, REG_BINARY, REG_DWORD, REG_MULTI_SZ, REG_QWORD].into_iter().find(|t| t.clone() as u32 == n)
    }

    /// Turns files off: saves their non-FontCraft values (before anything else), unloads them,
    /// deletes the values. Returns how many files were on.
    pub fn turn_off_in(key: &str, store: &Path, paths: &[PathBuf]) -> Result<usize, String> {
        if let Some(p) = paths.iter().find(|p| is_system_font(p)) {
            return Err(system_error(p));
        }
        let k = match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(key, KEY_READ | KEY_SET_VALUE) {
            Ok(k) => k,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(format!("can't open the font list: {e}")),
        };
        let found: Vec<(&PathBuf, Vec<(String, RegValue<'static>)>)> =
            paths.iter().map(|p| (p, values_for(&k, p))).filter(|(_, v)| !v.is_empty()).collect();
        let mut saved = Saved::load(store);
        let before = saved.entries.len();
        for (name, v) in found.iter().flat_map(|(_, vals)| vals) {
            if !name.ends_with(OWN_SUFFIX) {
                saved.add(Entry { name: name.clone(), vtype: v.vtype.clone() as u32, bytes: v.bytes.to_vec() });
            }
        }
        if saved.entries.len() != before {
            saved.save(store)?; // saved first: a value is never deleted before it's safe on disk
        }
        for (path, vals) in &found {
            // SAFETY: NUL-terminated buffer. A 0 return is fine: it may not be loaded, or be gone.
            unsafe { RemoveFontResourceW(wide(path).as_ptr()) };
            for (name, _) in vals {
                k.delete_value(name).map_err(|e| err("can't turn off", path, e))?;
            }
        }
        if !found.is_empty() {
            broadcast();
        }
        Ok(found.len())
    }

    /// Turns files on: puts saved values back byte for byte, or (for fonts with no saved values,
    /// e.g. from an added folder) writes a FontCraft marker. Returns how many files were off.
    pub fn turn_on_in(key: &str, store: &Path, paths: &[PathBuf]) -> Result<usize, String> {
        if let Some(p) = paths.iter().find(|p| is_system_font(p)) {
            return Err(system_error(p));
        }
        if let Some(missing) = paths.iter().find(|p| !p.is_file()) {
            return Err(format!("{}: file not found", missing.display()));
        }
        let (k, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(key).map_err(|e| format!("can't open the font list: {e}"))?;
        let mut saved = Saved::load(store);
        let mut turned_on = 0;
        for path in paths {
            let entries = saved.for_path(path);
            if entries.is_empty() {
                if values_for(&k, path).is_empty() {
                    turned_on += activate_in(key, std::slice::from_ref(path))?;
                }
                continue;
            }
            // SAFETY: NUL-terminated buffer that outlives the call.
            if unsafe { AddFontResourceW(wide(path).as_ptr()) } == 0 {
                return Err(format!("{}: Windows couldn't load this font", path.display()));
            }
            for e in &entries {
                let vtype = reg_type(e.vtype).ok_or_else(|| err("unknown saved value type for", path, e.vtype))?;
                let value = RegValue { bytes: e.bytes.clone().into(), vtype };
                k.set_raw_value(&e.name, &value).map_err(|er| err("can't turn on", path, er))?;
            }
            saved.remove(&entries);
            saved.save(store)?;
            turned_on += 1;
        }
        if turned_on > 0 {
            broadcast();
        }
        Ok(turned_on)
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

    /// Removes FontCraft markers only (turn_off_in does this as part of its work); kept for tests.
    #[cfg(test)]
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
pub use imp::{read_active, read_disabled, turn_off_in, turn_on_in};

#[cfg(unix)]
pub use crate::activate_unix::{read_active, read_disabled, turn_off_in, turn_on_in};

#[cfg(test)]
mod portable_tests {
    use super::*;

    #[test]
    fn link_name_differs_for_same_file_in_two_folders() {
        let a = link_name(Path::new("/a/Foo.ttf"));
        assert_ne!(a, link_name(Path::new("/b/Foo.ttf")));
        let hex = a.strip_prefix("Foo-").and_then(|r| r.strip_suffix(".ttf")).unwrap();
        assert!(hex.len() == 8 && hex.chars().all(|c| c.is_ascii_hexdigit()), "{a}");
    }

    #[test]
    fn lock_reason_system_font_is_some() {
        // /usr/share/fonts on Linux, /System/Library/Fonts on macOS, %WINDIR%\Fonts on Windows.
        let dir = crate::platform::system_font_dirs().into_iter().next().unwrap();
        assert!(lock_reason(&dir.join("x.ttf")).is_some());
    }
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

    impl Scratch {
        /// The saved list for this test (inside the temp folder).
        pub(crate) fn store(&self) -> PathBuf {
            self.dir.join("disabled.tsv")
        }

        fn key(&self) -> RegKey {
            RegKey::predef(HKEY_CURRENT_USER).create_subkey(&self.key).unwrap().0
        }

        /// A value as Windows writes it for a per-user font: REG_SZ full path.
        pub(crate) fn install(&self, name: &str, path: &Path) {
            self.key().set_value(name, &path.as_os_str()).unwrap();
        }

        fn raw(&self, name: &str) -> Option<winreg::RegValue<'static>> {
            self.key().get_raw_value(name).ok()
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let files: Vec<PathBuf> = std::fs::read_dir(&self.dir).into_iter().flatten().flatten().map(|e| e.path()).collect();
            let _ = deactivate_in(&self.key, &files);
            for f in &files {
                // Release every load the test made so the temp folder can be deleted.
                let w: Vec<u16> = { use std::os::windows::ffi::OsStrExt; f.as_os_str().encode_wide().chain(Some(0)).collect() };
                for _ in 0..8 {
                    if unsafe { windows_sys::Win32::Graphics::Gdi::RemoveFontResourceW(w.as_ptr()) } == 0 {
                        break;
                    }
                }
            }
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let _ = hkcu.delete_subkey_all(&self.key);
            // Fails while other tests still have keys under it; the last one removes it.
            let _ = hkcu.delete_subkey(r"Software\FontCraftTest");
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
        assert!(read_active(&s.key).contains(&norm(&f)));
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
        assert!(read_active(&s.key).contains(&norm(&f)));
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
        assert!(read_active(&s.key).contains(&norm(&f)));
        assert!(is_owned(&s.key, &f));
    }

    #[test]
    fn activate_makes_font_active_and_loads_it() {
        let s = Scratch::new("activate");
        let f = s.font("act.ttf");
        assert_eq!(activate_in(&s.key, &[f.clone()]), Ok(1));
        assert!(read_active(&s.key).contains(&norm(&f)));
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

    fn saved(s: &Scratch) -> Vec<crate::disabled::Entry> {
        crate::disabled::Saved::load(&s.store()).entries
    }

    #[test]
    fn user_installed_off_then_on_restores_value_exactly() {
        let s = Scratch::new("userround");
        let f = s.font("user.ttf");
        s.install("Foo Regular (TrueType)", &f);
        let before = s.raw("Foo Regular (TrueType)").unwrap();
        assert_eq!(turn_off_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert!(s.raw("Foo Regular (TrueType)").is_none());
        assert_eq!(saved(&s).len(), 1);
        assert!(!read_active(&s.key).contains(&norm(&f)));
        assert!(read_disabled(&s.store()).contains(&norm(&f)));
        assert_eq!(turn_on_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert!(s.raw("Foo Regular (TrueType)") == Some(before), "value must come back byte for byte");
        assert!(saved(&s).is_empty());
        assert!(read_active(&s.key).contains(&norm(&f)));
    }

    /// The scanner may hand over "…\Microsoft/Windows/Fonts\x.ttf" while the registry holds
    /// "…\Microsoft\Windows\Fonts\x.ttf": same file, so the saved value must still come back.
    #[test]
    fn mixed_slashes_still_restore_the_saved_value() {
        let s = Scratch::new("slashes");
        let f = s.font("slash.ttf");
        s.install("Slash Regular (TrueType)", &f);
        let before = s.raw("Slash Regular (TrueType)").unwrap();
        let mixed = PathBuf::from(f.to_string_lossy().replacen('\\', "/", 2));
        assert_ne!(mixed.as_os_str(), f.as_os_str());
        assert_eq!(turn_off_in(&s.key, &s.store(), &[mixed.clone()]), Ok(1));
        assert!(read_disabled(&s.store()).contains(&norm(&mixed)));
        assert_eq!(turn_on_in(&s.key, &s.store(), &[mixed.clone()]), Ok(1));
        assert!(s.raw("Slash Regular (TrueType)") == Some(before), "original value must come back");
        assert!(!is_owned(&s.key, &mixed), "no FontCraft marker instead of the original");
        assert!(saved(&s).is_empty());
        assert!(read_active(&s.key).contains(&norm(&mixed)));
    }

    #[test]
    fn fontbase_value_off_then_on_restores_it() {
        let s = Scratch::new("fbround");
        let f = s.font("fb.otf");
        s.install("Bar (FontBase)\n", &f);
        let before = s.raw("Bar (FontBase)\n").unwrap();
        assert_eq!(turn_off_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert!(s.raw("Bar (FontBase)\n").is_none());
        assert_eq!(turn_on_in(&s.key, &s.store(), &[f]), Ok(1));
        assert!(s.raw("Bar (FontBase)\n") == Some(before));
    }

    #[test]
    fn off_twice_keeps_one_saved_value() {
        let s = Scratch::new("offtwice");
        let f = s.font("twice.ttf");
        s.install("Twice (TrueType)", &f);
        assert_eq!(turn_off_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert_eq!(turn_off_in(&s.key, &s.store(), &[f.clone()]), Ok(0));
        assert_eq!(saved(&s).len(), 1);
        assert_eq!(turn_on_in(&s.key, &s.store(), &[f]), Ok(1));
        assert!(s.raw("Twice (TrueType)").is_some());
    }

    #[test]
    fn off_with_deleted_file_still_saves_the_value() {
        let s = Scratch::new("offgone");
        let f = s.dir.join("gone.ttf");
        s.install("Gone (TrueType)", &f);
        assert_eq!(turn_off_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert_eq!(saved(&s).len(), 1);
        assert!(s.raw("Gone (TrueType)").is_none());
        // Turning on a missing file is refused and changes nothing.
        assert!(turn_on_in(&s.key, &s.store(), &[f]).unwrap_err().contains("gone.ttf"));
        assert_eq!(saved(&s).len(), 1);
    }

    #[test]
    fn corrupt_saved_list_does_not_panic_and_is_kept() {
        let s = Scratch::new("corrupt");
        std::fs::write(s.store(), "not a valid line\n").unwrap();
        let f = s.font("c.ttf");
        s.install("C (TrueType)", &f);
        assert_eq!(turn_off_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert!(std::fs::read_to_string(s.store()).unwrap().contains("not a valid line"));
        assert_eq!(turn_on_in(&s.key, &s.store(), &[f]), Ok(1));
    }

    #[test]
    fn arabic_path_and_name_round_trip() {
        let s = Scratch::new("arabic");
        let f = s.font("خط تجريبي.ttf");
        s.install("خط تجريبي عادي (TrueType)", &f);
        let before = s.raw("خط تجريبي عادي (TrueType)").unwrap();
        assert_eq!(turn_off_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert_eq!(turn_on_in(&s.key, &s.store(), &[f]), Ok(1));
        assert!(s.raw("خط تجريبي عادي (TrueType)") == Some(before));
    }

    #[test]
    fn system_font_is_refused_without_touching_anything() {
        let s = Scratch::new("system");
        let arial = PathBuf::from(std::env::var_os("WINDIR").unwrap()).join(r"Fonts\arial.ttf");
        assert!(is_system_font(&arial));
        let e = turn_off_in(&s.key, &s.store(), &[arial.clone()]).unwrap_err();
        assert!(e.contains("needs administrator"), "{e}");
        assert!(turn_on_in(&s.key, &s.store(), &[arial]).unwrap_err().contains("needs administrator"));
        assert_eq!(s.value_count(), 0);
        assert!(!s.store().exists());
    }

    #[test]
    fn added_folder_font_turns_on_with_a_fontcraft_marker_and_off_without_saving() {
        let s = Scratch::new("added");
        let f = s.font("added.ttf");
        assert_eq!(turn_on_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert!(is_owned(&s.key, &f));
        assert_eq!(turn_off_in(&s.key, &s.store(), &[f.clone()]), Ok(1));
        assert!(!is_owned(&s.key, &f));
        assert!(saved(&s).is_empty(), "FontCraft's own marker is not saved");
    }

    /// Live check of turning off and on a user-installed-style value on the real key; run with
    /// `--ignored`. Uses a temp copy of Arial and a temp saved list; removes everything it added.
    #[test]
    #[ignore]
    fn live_user_installed_off_and_on_on_real_key() {
        let s = Scratch::new("liveuser");
        let f = s.font("FontCraftLiveUser.ttf");
        let name = "FontCraft Live User Test (TrueType)";
        let real = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(super::FONTS_KEY, winreg::enums::KEY_ALL_ACCESS).unwrap();
        real.set_value(name, &f.as_os_str()).unwrap();
        let before = real.get_raw_value(name).unwrap();
        let off = turn_off_in(super::FONTS_KEY, &s.store(), &[f.clone()]);
        let gone = real.get_raw_value(name).is_err();
        let on = turn_on_in(super::FONTS_KEY, &s.store(), &[f.clone()]);
        let back = real.get_raw_value(name).ok();
        let _ = real.delete_value(name);
        let _ = deactivate_in(super::FONTS_KEY, &[f]);
        assert_eq!(off, Ok(1));
        assert!(gone, "value still there after turn off");
        assert_eq!(on, Ok(1));
        assert!(back == Some(before), "value didn't come back byte for byte");
        println!("live user-installed ok");
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
        assert!(read_active(&s.key).contains(&norm(&f)));
        assert_eq!(deactivate_in(&s.key, &[f]), Ok(1));
    }
}
