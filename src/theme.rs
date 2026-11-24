//! Design tokens. The only file that builds colors from raw RGB.
use eframe::egui::{self, Color32};

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
    #[allow(dead_code)] // part of the token set; no success state in the UI yet
    pub const SUCCESS: Color32 = hex(0x2F, 0x7D, 0x4F);
}

pub mod space {
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
    #[allow(dead_code)] // top of the spacing scale
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
    #[allow(dead_code)] // section title size, used by the next view
    pub const TITLE: f32 = 15.0;
    pub const DETAIL: f32 = 20.0;
}

pub const HEADER_HEIGHT: f32 = 52.0;
pub const ROW_HEIGHT: f32 = 32.0;

#[cfg(test)] // only the contrast test measures colours
fn luminance(c: Color32) -> f32 {
    let lin = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
}

/// WCAG contrast ratio, 1.0 to 21.0, order-independent.
#[cfg(test)]
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Archivo Medium, for fixed English labels only.
pub fn medium() -> egui::FontFamily {
    egui::FontFamily::Name("ui-medium".into())
}

/// IBM Plex Mono.
pub fn mono() -> egui::FontFamily {
    egui::FontFamily::Name("ui-mono".into())
}

/// Registers the UI faces and applies the design system. Call once, at startup.
pub fn apply(ctx: &egui::Context) {
    register_fonts(ctx);
    ctx.all_styles_mut(|style| {
        style.text_styles = text_styles();
        style.spacing = spacing();
        style.visuals = paper_visuals();
    });
}

fn register_fonts(ctx: &egui::Context) {
    let inserts: [(&str, &'static [u8], egui::FontFamily); 3] = [
        ("Archivo", include_bytes!("../assets/fonts/Archivo-Regular.ttf") as &[u8], egui::FontFamily::Proportional),
        ("ui-medium", include_bytes!("../assets/fonts/Archivo-Medium.ttf"), medium()),
        ("ui-mono", include_bytes!("../assets/fonts/PlexMono-Regular.ttf"), mono()),
    ];
    for (name, bytes, family) in inserts {
        let family = egui::InsertFontFamily { family, priority: egui::FontPriority::Highest };
        ctx.add_font(egui::FontInsert::new(name, egui::FontData::from_static(bytes), vec![family]));
    }
}

fn text_styles() -> std::collections::BTreeMap<egui::TextStyle, egui::FontId> {
    use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
    [
        (Small, egui::FontId::proportional(size::META)),
        (Body, egui::FontId::proportional(size::BODY)),
        (Button, egui::FontId::proportional(size::BODY)),
        (Heading, egui::FontId::proportional(size::DETAIL)),
        (Monospace, egui::FontId::new(size::BODY, mono())),
    ]
    .into()
}

fn spacing() -> egui::Spacing {
    let mut s = egui::Spacing::default();
    s.item_spacing = egui::vec2(space::S, space::XS);
    s.button_padding = egui::vec2(space::M, space::XS);
    s.menu_margin = egui::Margin::same(space::S as i8);
    s.window_margin = egui::Margin::same(space::M as i8);
    s.indent = space::L;
    s.icon_spacing = space::XS;
    s
}

fn widget(state: &mut egui::style::WidgetVisuals, bg: Color32, line: Color32, text: Color32) {
    state.bg_fill = bg;
    state.weak_bg_fill = bg;
    state.bg_stroke = egui::Stroke::new(1.0, line);
    state.fg_stroke = egui::Stroke::new(1.0, text);
    state.corner_radius = egui::CornerRadius::same(radius::CONTROL as u8);
    state.expansion = 0.0;
}

fn widgets(base: Color32, fill: Color32, hover: Color32, pressed: Color32, line: Color32, text: Color32) -> egui::style::Widgets {
    let mut w = egui::style::Widgets::default();
    widget(&mut w.noninteractive, base, line, text);
    widget(&mut w.inactive, fill, line, text);
    widget(&mut w.hovered, hover, line, text);
    widget(&mut w.active, pressed, line, text);
    widget(&mut w.open, hover, line, text);
    w
}

fn paper_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::light();
    v.override_text_color = Some(color::INK);
    v.panel_fill = color::PAPER;
    v.window_fill = color::CARD;
    v.faint_bg_color = color::CANVAS_HOVER;
    v.extreme_bg_color = color::CARD;
    v.code_bg_color = color::CANVAS_HOVER;
    v.window_stroke = egui::Stroke::new(1.0, color::CANVAS_LINE);
    v.selection.bg_fill = color::COBALT;
    v.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
    v.hyperlink_color = color::COBALT;
    v.window_shadow = egui::Shadow::NONE;
    v.popup_shadow = egui::Shadow::NONE;
    v.widgets = widgets(color::PAPER, color::CARD, color::CANVAS_HOVER, color::SELECTED, color::CANVAS_LINE, color::INK);
    v
}

/// Dark controls for the header and sidebar, with the accent for selection.
pub fn chrome_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(color::CHROME_TEXT);
    v.panel_fill = color::SIDEBAR;
    v.window_fill = color::HEADER;
    v.faint_bg_color = color::CHROME_HOVER;
    v.extreme_bg_color = color::HEADER;
    v.code_bg_color = color::CHROME_HOVER;
    v.window_stroke = egui::Stroke::new(1.0, color::CHROME_LINE);
    v.selection.bg_fill = color::ACCENT_ON_DARK;
    v.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
    v.hyperlink_color = color::ACCENT_ON_DARK;
    v.window_shadow = egui::Shadow::NONE;
    v.popup_shadow = egui::Shadow::NONE;
    v.widgets = widgets(color::HEADER, color::CHROME_RAISED, color::CHROME_HOVER, color::CHROME_RAISED, color::CHROME_LINE, color::CHROME_TEXT);
    v
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
    #[test] fn apply_sets_the_text_scale() {
        let ctx = egui::Context::default();
        apply(&ctx);
        let s = ctx.global_style();
        assert_eq!(s.text_styles[&egui::TextStyle::Body].size, size::BODY);
        assert_eq!(s.text_styles[&egui::TextStyle::Heading].size, size::DETAIL);
        assert_eq!(s.text_styles[&egui::TextStyle::Small].size, size::META);
    }
    #[test] fn arabic_still_reaches_a_fallback_font() {
        // only meaningful where the OS has an Arabic font to borrow
        if !cfg!(windows) { return; }
        let ctx = egui::Context::default();
        apply(&ctx);
        crate::fonts::add_system_fallback(&ctx);
        let mut out = ctx.run_pass(Default::default(), |_| {}); // fonts take effect on the next frame
        out.textures_delta.clear();
        let ok = ctx.fonts_mut(|f| f.has_glyphs(&egui::FontId::proportional(13.0), "مرحبا"));
        assert!(ok);
    }
}
