//! The "Buy me a coffee" support link and the app picture (window icon and support card).
use std::process::Command;

use eframe::egui;

pub const SUPPORT_URL: &str = "https://buymeacoffee.com/mohseenjamalll";
const ART: &[u8] = include_bytes!("../assets/app-art.png");

/// The picture decoded to RGBA (256x256).
fn art() -> image::RgbaImage {
    image::load_from_memory_with_format(ART, image::ImageFormat::Png).expect("app-art.png is a valid PNG").to_rgba8()
}

pub fn app_icon() -> egui::IconData {
    let img = art();
    egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() }
}

pub fn art_image() -> egui::ColorImage {
    let img = art();
    egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw())
}

/// Opens the support page (and only that) in the default browser.
pub fn open_support() -> Result<(), String> {
    let opener = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(opener).arg(SUPPORT_URL).spawn().map(drop).map_err(|e| format!("couldn't open the support page: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_url_is_the_buymeacoffee_page() {
        assert_eq!(SUPPORT_URL, concat!("https://buymeacoffee.com/", "mohseenjamalll"));
    }

    #[test]
    fn app_icon_is_256_square_rgba() {
        let icon = app_icon();
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
    }
}
