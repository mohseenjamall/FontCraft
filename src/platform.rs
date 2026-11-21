//! Every OS-specific path in one place: where settings live, where fonts are installed or loaded from.
use std::path::{Path, PathBuf};

/// XDG rule: use `xdg` only if absolute, else `home/home_rel`, else nothing (never a relative path).
pub fn linux_dir(home: Option<&Path>, xdg: Option<&Path>, home_rel: &str) -> Option<PathBuf> {
    match xdg {
        Some(x) if x.is_absolute() => Some(x.to_path_buf()),
        _ => home.map(|h| h.join(home_rel)),
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from)
}

fn xdg(var: &str, home_rel: &str) -> Option<PathBuf> {
    linux_dir(env_path("HOME").as_deref(), env_path(var).as_deref(), home_rel)
}

/// Where FontCraft keeps its own settings.
pub fn app_data_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        env_path("APPDATA").map(|d| d.join("FontCraft"))
    } else if cfg!(target_os = "macos") {
        env_path("HOME").map(|h| h.join("Library/Application Support/FontCraft"))
    } else {
        xdg("XDG_CONFIG_HOME", ".config").map(|d| d.join("FontCraft"))
    }
}

/// Where fonts installed for the current user live.
pub fn user_fonts_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        env_path("LOCALAPPDATA").map(|d| d.join(r"Microsoft\Windows\Fonts"))
    } else if cfg!(target_os = "macos") {
        env_path("HOME").map(|h| h.join("Library/Fonts"))
    } else {
        xdg("XDG_DATA_HOME", ".local/share").map(|d| d.join("fonts"))
    }
}

/// Folders the system loads fonts from.
pub fn system_font_dirs() -> Vec<PathBuf> {
    if cfg!(windows) {
        env_path("WINDIR").map(|w| w.join("Fonts")).into_iter().collect()
    } else if cfg!(target_os = "macos") {
        vec!["/System/Library/Fonts".into(), "/Library/Fonts".into()]
    } else {
        vec!["/usr/share/fonts".into(), "/usr/local/share/fonts".into()]
    }
}

/// System fonts with Arabic/Hebrew glyphs (egui's built-ins have none) that exist on this machine.
pub fn fallback_fonts() -> Vec<PathBuf> {
    let found: Vec<PathBuf> = if cfg!(windows) {
        let fonts = env_path("WINDIR").map(|w| w.join("Fonts")).unwrap_or_default();
        vec![fonts.join("segoeui.ttf"), fonts.join("tahoma.ttf")]
    } else if cfg!(target_os = "macos") {
        vec![
            "/System/Library/Fonts/Supplemental/Arial.ttf".into(),
            "/System/Library/Fonts/Supplemental/Tahoma.ttf".into(),
        ]
    } else {
        ["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "/usr/share/fonts/TTF/DejaVuSans.ttf", "/usr/share/fonts/noto/NotoSansArabic-Regular.ttf"]
            .into_iter()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .take(1)
            .collect()
    };
    found.into_iter().filter(|p| p.exists()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // A path that is absolute on every OS (so the tests run on Windows too).
    fn abs(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }

    #[test]
    fn linux_dir_prefers_absolute_xdg() {
        let x = abs("data");
        assert_eq!(linux_dir(Some(&abs("home")), Some(&x), ".local/share"), Some(x));
    }

    #[test]
    fn linux_dir_ignores_relative_xdg() {
        let h = abs("home");
        assert_eq!(linux_dir(Some(&h), Some(Path::new("rel")), ".local/share"), Some(h.join(".local/share")));
    }

    #[test]
    fn linux_dir_none_without_home_or_xdg() {
        assert_eq!(linux_dir(None, None, ".config"), None);
        assert_eq!(linux_dir(None, Some(Path::new("rel")), ".config"), None);
    }

    #[test]
    fn linux_dir_appends_nothing_when_xdg_used() {
        let x = abs("x");
        assert_eq!(linux_dir(Some(&abs("h")), Some(&x), ".config"), Some(x));
    }
}
