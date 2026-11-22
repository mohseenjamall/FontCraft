//! Design tokens. The only file that builds colors from raw RGB.
use eframe::egui::Color32;

pub mod color {
    use super::Color32;
    const fn hex(r: u8, g: u8, b: u8) -> Color32 { Color32::from_rgb(r, g, b) }
    pub const HEADER: Color32 = hex(0x10, 0x12, 0x16);
    pub const SIDEBAR: Color32 = hex(0x15, 0x17, 0x1B);
    pub const CHROME_RAISED: Color32 = hex(0x1F, 0x23, 0x30);
    pub const CHROME_HOVER: Color32 = hex(0x1A, 0x1D, 0x23);
    pub const CHROME_LINE: Color32 = hex(0x2A, 0x2D, 0x33);
    pub const CHROME_TEXT: Color32 = hex(0xE8, 0xEA, 0xEE);
    pub const CHROME_TEXT_DIM: Color32 = hex(0xA3, 0xA8, 0xB1);
    pub const CHROME_TEXT_MUTED: Color32 = hex(0x7B, 0x81, 0x8B);
    pub const ACCENT_ON_DARK: Color32 = hex(0x6A, 0x85, 0xFF);
    pub const PAPER: Color32 = hex(0xF7, 0xF5, 0xF0);
    pub const CARD: Color32 = hex(0xFF, 0xFF, 0xFF);
    pub const SELECTED: Color32 = hex(0xE4, 0xEA, 0xFB);
    pub const CANVAS_HOVER: Color32 = hex(0xEF, 0xEC, 0xE5);
    pub const CANVAS_LINE: Color32 = hex(0xE3, 0xDF, 0xD6);
    pub const INK: Color32 = hex(0x1B, 0x1C, 0x1F);
    pub const INK_DIM: Color32 = hex(0x5B, 0x5F, 0x66);
    pub const INK_MUTED: Color32 = hex(0x6C, 0x70, 0x77);
    pub const COBALT: Color32 = hex(0x1A, 0x4A, 0xC4);
    pub const COBALT_HOVER: Color32 = hex(0x24, 0x57, 0xD6);
    pub const COBALT_PRESSED: Color32 = hex(0x17, 0x3F, 0xA8);
    pub const DANGER: Color32 = hex(0xB4, 0x23, 0x18);
    pub const SUCCESS: Color32 = hex(0x2F, 0x7D, 0x4F);
}

pub mod space {
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

/// f32: egui's `corner_radius(8.0)` call sites in app.rs pass floats.
pub mod radius {
    pub const CONTROL: f32 = 8.0;
    pub const CARD: f32 = 12.0;
}

pub mod size {
    pub const META: f32 = 11.0;
    pub const SECONDARY: f32 = 12.0;
    pub const BODY: f32 = 13.0;
    pub const TITLE: f32 = 15.0;
    pub const DETAIL: f32 = 20.0;
}

pub const HEADER_HEIGHT: f32 = 52.0;
pub const ROW_HEIGHT: f32 = 32.0;

fn luminance(c: Color32) -> f32 {
    let lin = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
}

/// WCAG contrast ratio, 1.0 to 21.0, order-independent.
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Color32;

    #[test] fn contrast_of_black_on_white_is_21() { assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01); }
    #[test] fn text_roles_reach_4_5_on_their_surface() {
        use color::*;
        for (fg, bg) in [(CHROME_TEXT, SIDEBAR), (CHROME_TEXT, HEADER), (CHROME_TEXT_DIM, SIDEBAR),
            (CHROME_TEXT_MUTED, SIDEBAR), (ACCENT_ON_DARK, SIDEBAR), (INK, PAPER), (INK_DIM, PAPER),
            (INK_MUTED, PAPER), (INK_MUTED, CARD), (COBALT, PAPER), (Color32::WHITE, COBALT),
            (Color32::WHITE, COBALT_HOVER), (INK, SELECTED), (DANGER, CARD)] {
            assert!(contrast(fg, bg) >= 4.5, "{fg:?} on {bg:?} = {}", contrast(fg, bg));
        }
    }
    #[test] fn hairlines_are_visible_but_quiet() {
        use color::*;
        assert!(contrast(CHROME_LINE, SIDEBAR) >= 1.2 && contrast(CANVAS_LINE, PAPER) >= 1.1);
    }
}
