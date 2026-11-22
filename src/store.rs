//! Remembers the folders the person added, one path per line, in the per-OS folder from `platform::app_data_dir` (%APPDATA%\FontCraft on Windows).
use std::path::PathBuf;

fn dir() -> Option<PathBuf> {
    crate::platform::app_data_dir()
}

fn file() -> Option<PathBuf> {
    dir().map(|d| d.join("folders.txt"))
}

/// Where turned-off fonts are recorded: on Windows their saved registry values (see `disabled.rs`),
/// on Linux FontCraft's fontconfig reject file (see `activate_unix.rs`); unused on macOS.
pub fn disabled_list() -> PathBuf {
    let path = if cfg!(any(windows, target_os = "macos")) {
        dir().map(|d| d.join("disabled.tsv"))
    } else {
        crate::platform::xdg("XDG_CONFIG_HOME", ".config").map(|d| d.join("fontconfig/conf.d/99-fontcraft.conf"))
    };
    // An empty path makes saving fail with a clear error instead of writing somewhere unexpected.
    path.unwrap_or_default()
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
