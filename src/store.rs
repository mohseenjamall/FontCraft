//! Remembers the folders the person added, one path per line.
use std::path::PathBuf;

fn file() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("FontCraft").join("folders.txt"))
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
