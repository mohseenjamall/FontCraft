//! Remembers the folders the person added, one path per line, in %APPDATA%\FontCraft.
use std::path::PathBuf;

fn dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("FontCraft"))
}

fn file() -> Option<PathBuf> {
    dir().map(|d| d.join("folders.txt"))
}

/// Where turned-off fonts' registry values are kept (see `disabled.rs`).
pub fn disabled_list() -> PathBuf {
    // An empty path makes saving fail with a clear error instead of writing somewhere unexpected.
    dir().map(|d| d.join("disabled.tsv")).unwrap_or_default()
}

pub fn load_folders() -> Vec<PathBuf> {
    let Some(f) = file() else { return Vec::new() };
    std::fs::read_to_string(f)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .collect()
}

pub fn save_folders(folders: &[PathBuf]) {
    let Some(f) = file() else { return };
    let text: String = folders.iter().map(|p| format!("{}\n", p.display())).collect();
    let saved = f.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(&f, text));
    if let Err(e) = saved {
        eprintln!("could not remember folders in {}: {e}", f.display());
    }
}
