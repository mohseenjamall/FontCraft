//! Registry values FontCraft took out of the user's font list when turning a font off, kept so
//! Turn on can put them back byte for byte. One value per line: `type<TAB>hex(name)<TAB>hex(bytes)`.
//! Lines that can't be read are kept as they are, so rewriting the file never loses anything.
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub vtype: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct Saved {
    pub entries: Vec<Entry>,
    /// Lines that didn't parse, written back unchanged.
    unreadable: Vec<String>,
}

impl Entry {
    /// The font file this value points at (REG_SZ / REG_EXPAND_SZ only).
    pub fn path(&self) -> Option<PathBuf> {
        data_path(self.vtype, &self.bytes)
    }
}

/// Decodes a font registry value's data into a file path. Relative names (as Windows writes for
/// fonts in the Fonts folder itself) resolve against the per-user fonts folder.
/// shortcut: %VARS% in REG_EXPAND_SZ aren't expanded; Windows writes full paths for per-user fonts.
pub fn data_path(vtype: u32, bytes: &[u8]) -> Option<PathBuf> {
    const REG_SZ: u32 = 1;
    const REG_EXPAND_SZ: u32 = 2;
    if vtype != REG_SZ && vtype != REG_EXPAND_SZ || bytes.len() % 2 != 0 {
        return None;
    }
    let wide: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    let text = String::from_utf16(&wide).ok()?;
    let text = text.trim_end_matches('\0');
    if text.is_empty() {
        return None;
    }
    let p = PathBuf::from(text);
    if p.is_absolute() {
        return Some(p);
    }
    std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join(r"Microsoft\Windows\Fonts").join(p))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

fn parse(line: &str) -> Option<Entry> {
    let mut parts = line.split('\t');
    let vtype = parts.next()?.parse().ok()?;
    let name = String::from_utf8(unhex(parts.next()?)?).ok()?;
    let bytes = unhex(parts.next()?)?;
    parts.next().is_none().then_some(Entry { name, vtype, bytes })
}

impl Saved {
    /// A missing or unreadable file reads as empty; it never panics.
    pub fn load(file: &Path) -> Self {
        let text = std::fs::read(file).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
        let mut saved = Saved::default();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            match parse(line) {
                Some(e) => saved.entries.push(e),
                None => saved.unreadable.push(line.to_owned()),
            }
        }
        saved
    }

    /// Writes to a temp file next to it, then renames, so a crash can't leave half a file.
    pub fn save(&self, file: &Path) -> Result<(), String> {
        let mut text: String = self.entries.iter().map(|e| format!("{}\t{}\t{}\n", e.vtype, hex(e.name.as_bytes()), hex(&e.bytes))).collect();
        for line in &self.unreadable {
            text.push_str(line);
            text.push('\n');
        }
        let tmp = file.with_extension("tmp");
        file.parent().map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&tmp, text))
            .and_then(|()| std::fs::rename(&tmp, file))
            .map_err(|e| format!("can't save the list of turned-off fonts in {}: {e}", file.display()))
    }

    /// Adds an entry unless an identical one is already saved.
    pub fn add(&mut self, entry: Entry) {
        if !self.entries.contains(&entry) {
            self.entries.push(entry);
        }
    }

    /// Entries whose data points at `path` (case-insensitive, as Windows compares paths).
    pub fn for_path(&self, path: &Path) -> Vec<Entry> {
        let want = path.to_string_lossy().to_lowercase();
        self.entries.iter().filter(|e| e.path().is_some_and(|p| p.to_string_lossy().to_lowercase() == want)).cloned().collect()
    }

    pub fn remove(&mut self, gone: &[Entry]) {
        self.entries.retain(|e| !gone.contains(e));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(s: &str) -> Vec<u8> {
        s.encode_utf16().chain(Some(0)).flat_map(u16::to_le_bytes).collect()
    }

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("fontcraft-saved-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("disabled.tsv")
    }

    #[test]
    fn missing_file_reads_as_empty() {
        let s = Saved::load(Path::new(r"Z:\definitely\not\here\disabled.tsv"));
        assert!(s.entries.is_empty());
    }

    #[test]
    fn roundtrip_keeps_name_type_and_bytes_exactly() {
        let f = tmp("roundtrip");
        let e = Entry { name: "خط عربي (FontBase)\n".into(), vtype: 2, bytes: utf16(r"C:\خطوط\a.ttf") };
        let mut s = Saved::default();
        s.add(e.clone());
        s.add(e.clone());
        s.save(&f).unwrap();
        let back = Saved::load(&f);
        assert_eq!(back.entries, vec![e]);
        let _ = std::fs::remove_dir_all(f.parent().unwrap());
    }

    #[test]
    fn corrupt_lines_never_panic_and_survive_a_rewrite() {
        let f = tmp("corrupt");
        std::fs::write(&f, b"garbage line\n1\tzz\t00\n\xff\xfe\x00\n").unwrap();
        let mut s = Saved::load(&f);
        assert!(s.entries.is_empty());
        s.add(Entry { name: "A (TrueType)".into(), vtype: 1, bytes: utf16(r"C:\x\a.ttf") });
        s.save(&f).unwrap();
        let text = String::from_utf8_lossy(&std::fs::read(&f).unwrap()).into_owned();
        assert!(text.contains("garbage line"), "{text}");
        assert_eq!(Saved::load(&f).entries.len(), 1);
        let _ = std::fs::remove_dir_all(f.parent().unwrap());
    }

    #[test]
    fn for_path_matches_case_insensitively_and_resolves_relative_names() {
        let mut s = Saved::default();
        s.add(Entry { name: "A".into(), vtype: 1, bytes: utf16(r"C:\Fonts\A.TTF") });
        s.add(Entry { name: "B".into(), vtype: 1, bytes: utf16("b.ttf") });
        assert_eq!(s.for_path(Path::new(r"c:\fonts\a.ttf")).len(), 1);
        let user = PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap()).join(r"Microsoft\Windows\Fonts\b.ttf");
        assert_eq!(s.for_path(&user).len(), 1);
        assert_eq!(data_path(4, &[0, 0, 0, 0]), None);
    }
}
