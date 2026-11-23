//! Remembers the folders the person added, one path per line, and the list or grid choice,
//! in the per-OS folder from `platform::app_data_dir` (%APPDATA%\FontCraft on Windows).
use std::path::{Path, PathBuf};

use crate::grid::View;

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

/// The view stored at `path`; a missing or unknown value means the list.
pub fn read_view(path: &Path) -> View {
    std::fs::read_to_string(path).map_or(View::List, |s| View::parse(&s))
}

/// Writes `v` to `path`, creating its folder when needed.
pub fn write_view(path: &Path, v: View) {
    let saved = path.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(path, v.as_str()));
    if let Err(e) = saved {
        eprintln!("could not remember the view in {}: {e}", path.display());
    }
}

/// The view saved from last run, or the list when there is none.
pub fn load_view() -> View {
    crate::platform::app_data_dir().map_or(View::List, |d| read_view(&d.join("view.txt")))
}

/// Remembers the view for the next run.
pub fn save_view(v: View) {
    if let Some(d) = crate::platform::app_data_dir() {
        write_view(&d.join("view.txt"), v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_roundtrips_through_a_file() {
        let dir = std::env::temp_dir().join(format!("fontcraft_view_{}", std::process::id()));
        let path = dir.join("view.txt");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(read_view(&path), View::List, "a missing file is the list");
        write_view(&path, View::Grid);
        assert_eq!(read_view(&path), View::Grid);
        write_view(&path, View::List);
        assert_eq!(read_view(&path), View::List);
        std::fs::write(&path, "\u{0}garbage\n").unwrap();
        assert_eq!(read_view(&path), View::List, "an unknown value is the list");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
