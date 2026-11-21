//! Finds font files and reads their family / style names (no rendering here).
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct Face {
    pub path: PathBuf,
    pub index: u32,
    pub family: String,
    pub style: String,
    pub weight: u16,
    pub italic: bool,
    /// Index into samples::SCRIPTS: the writing system its default sample uses.
    pub script: u8,
    pub source: usize,
}

impl Face {
    /// Unique id used as the egui font name.
    pub fn key(&self) -> String {
        format!("{}#{}", self.path.display(), self.index)
    }
}

const EXTENSIONS: [&str; 4] = ["ttf", "otf", "ttc", "otc"];

/// Walks every folder (recursively) and sends each face it can read.
/// Files are split across threads because reading big CJK fonts dominates the time.
pub fn scan(folders: Vec<(usize, PathBuf)>, tx: Sender<Face>) {
    let mut files = Vec::new();
    for (source, dir) in &folders {
        collect_files(dir, *source, &mut files);
    }
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(8);
    let chunk = files.len().div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for part in files.chunks(chunk) {
            let tx = tx.clone();
            s.spawn(move || {
                for (source, path) in part {
                    for face in faces_in_file(path, *source) {
                        if tx.send(face).is_err() {
                            return; // window closed
                        }
                    }
                }
            });
        }
    });
}

fn collect_files(dir: &Path, source: usize, out: &mut Vec<(usize, PathBuf)>) {
    if dir.is_file() {
        out.push((source, dir.to_path_buf()));
        return;
    }
    walk(dir, source, out, &mut HashSet::new());
}

/// Recursive walk; `seen` holds canonical dirs so symlink loops end instead of recursing forever.
fn walk(dir: &Path, source: usize, out: &mut Vec<(usize, PathBuf)>, seen: &mut HashSet<PathBuf>) {
    let Ok(real) = std::fs::canonicalize(dir) else { return };
    if !seen.insert(real) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, source, out, seen);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        {
            out.push((source, path));
        }
    }
}

fn faces_in_file(path: &Path, source: usize) -> Vec<Face> {
    let Ok(data) = std::fs::read(path) else { return Vec::new() };
    let count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
    (0..count)
        .filter_map(|index| {
            let face = ttf_parser::Face::parse(&data, index).ok()?;
            let family = name(&face, 16).or_else(|| name(&face, 1))?;
            let style = name(&face, 17).or_else(|| name(&face, 2)).unwrap_or_else(|| "Regular".into());
            Some(Face {
                path: path.to_path_buf(),
                index,
                family,
                style,
                weight: face.weight().to_number(),
                italic: face.is_italic(),
                script: crate::samples::script_of(|c| face.glyph_index(c).is_some_and(|g| g.0 != 0)),
                source,
            })
        })
        .collect()
}

/// Reads one name-table entry, preferring US English.
fn name(face: &ttf_parser::Face, id: u16) -> Option<String> {
    let mut fallback = None;
    for n in face.names().into_iter().filter(|n| n.name_id == id) {
        if let Some(s) = n.to_string().filter(|s| !s.trim().is_empty()) {
            if n.language_id == 0x409 {
                return Some(s);
            }
            fallback.get_or_insert(s);
        }
    }
    fallback
}

/// Folders the system loads fonts from: (label, path).
pub fn system_folders() -> Vec<(String, PathBuf)> {
    let system = crate::platform::system_font_dirs().into_iter().map(|p| ("System".to_string(), p));
    let user = crate::platform::user_fonts_dir().map(|p| ("User installed".to_string(), p));
    system.chain(user).collect()
}

/// Font files FontBase turned on from anywhere on disk. Its native add-on (registerFonts,
/// 0x180001200) writes each one to the current user's font registry key as
/// "<name> (FontBase)\n" = <file path>, after AddFontResourceW.
#[cfg(windows)]
pub fn fontbase_activated() -> Vec<PathBuf> {
    use winreg::{RegKey, enums::HKEY_CURRENT_USER};
    let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Software\Microsoft\Windows NT\CurrentVersion\Fonts")
    else {
        return Vec::new();
    };
    key.enum_values()
        .flatten()
        .filter(|(name, _)| name.contains(" (FontBase)"))
        .filter_map(|(name, _)| key.get_value::<String, _>(&name).ok())
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .collect()
}

#[cfg(not(windows))]
pub fn fontbase_activated() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn reads_names_from_a_system_font() {
        let (_, dir) = system_folders().into_iter().next().expect("windows fonts folder");
        let faces = faces_in_file(&dir.join("arial.ttf"), 0);
        assert!(faces.iter().any(|f| f.family == "Arial" && f.weight == 400 && !f.italic));
    }

    #[cfg(unix)]
    #[test]
    fn scan_finishes_on_symlink_loop() {
        let root = std::env::temp_dir().join(format!("fontcraft_loop_{}", std::process::id()));
        let fonts = root.join("fonts");
        std::fs::create_dir_all(&fonts).unwrap();
        std::fs::write(fonts.join("a.ttf"), b"not a font").unwrap();
        std::os::unix::fs::symlink(&root, fonts.join("loop")).unwrap();

        let mut out = Vec::new();
        collect_files(&fonts, 0, &mut out);
        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(out.len(), 1);
        assert!(out[0].1.ends_with("a.ttf"));
    }
}
