//! Shared chrome widgets: buttons, a section label, the ruler slider and the status dot.
use std::ops::RangeInclusive;

use eframe::egui::{self, Color32, FontId, Response, RichText, Stroke};

use crate::theme::{self, color, radius, size, space};

/// Minor ruler tick: every 6 px of value.
const TICK_MINOR: f32 = 6.0;
/// Major ruler tick: every 30 px of value.
const TICK_MAJOR: f32 = 30.0;
const TICK_MINOR_HEIGHT: f32 = 3.0;
const TICK_MAJOR_HEIGHT: f32 = 6.0;
/// Hard cap so the ruler can never flood the frame with painter calls.
const TICK_LIMIT: usize = 36;

/// Cobalt with white text; hover and press use their own tokens.
pub fn primary_button(ui: &mut egui::Ui, label: &str) -> Response {
    button(
        ui,
        label,
        color::COBALT,
        color::COBALT_HOVER,
        color::COBALT_PRESSED,
        Stroke::NONE,
        Color32::WHITE,
    )
}

/// Transparent with a 1 px hairline; the fill rises on hover.
pub fn ghost_button(ui: &mut egui::Ui, label: &str) -> Response {
    button(
        ui,
        label,
        Color32::TRANSPARENT,
        color::CHROME_HOVER,
        color::CHROME_RAISED,
        Stroke::new(1.0, color::CHROME_LINE),
        color::CHROME_TEXT,
    )
}

/// Shared drawing for the two button kinds, so both keep the same metrics.
fn button(
    ui: &mut egui::Ui,
    label: &str,
    fill: Color32,
    hover: Color32,
    pressed: Color32,
    stroke: Stroke,
    text: Color32,
) -> Response {
    let font = FontId::new(size::BODY, theme::medium());
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font, text);
    let padding = egui::vec2(space::M, 6.0);
    let (rect, response) = ui.allocate_exact_size(galley.size() + 2.0 * padding, egui::Sense::click());
    let fill = if response.is_pointer_button_down_on() {
        pressed
    } else if response.hovered() {
        hover
    } else {
        fill
    };
    ui.painter().rect(rect, radius::CONTROL, fill, stroke, egui::StrokeKind::Inside);
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, text);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Small heading above a group of rows: 13 px medium, secondary colour.
pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).font(FontId::new(size::BODY, theme::medium())).color(color::CHROME_TEXT_DIM));
}

/// The normal egui slider with its value in px, plus a ruler under the track.
pub fn ruler_slider(ui: &mut egui::Ui, value: &mut f32, range: RangeInclusive<f32>, step: f64) -> Response {
    let response = ui.add(egui::Slider::new(value, range.clone()).step_by(step).suffix(" px"));
    // The slider and its value box share one response; the track is the slider's own width.
    let track = egui::Rect::from_min_size(
        response.rect.left_top(),
        egui::vec2(ui.spacing().slider_width, response.rect.height()),
    );
    let ruler = egui::Rect::from_min_max(
        egui::pos2(track.left(), track.bottom() + 1.0),
        egui::pos2(track.right(), track.bottom() + 1.0 + TICK_MAJOR_HEIGHT),
    );
    // Painted into the header's spare height without moving the cursor, so the next widget keeps its place.
    paint_ticks(ui.painter(), track, ruler, &range);
    response
}

fn paint_ticks(painter: &egui::Painter, track: egui::Rect, ruler: egui::Rect, range: &RangeInclusive<f32>) {
    let (min, max) = (*range.start(), *range.end());
    if max <= min {
        return;
    }
    // The handle centres are inset from the track ends by its radius, as the slider draws them.
    let inset = track.height() / 2.5;
    let span = track.width() - 2.0 * inset;
    if span <= 0.0 {
        return;
    }
    let stroke = Stroke::new(1.0, color::CHROME_LINE);
    let mut value = (min / TICK_MINOR).ceil() * TICK_MINOR;
    let mut drawn = 0;
    while value <= max + 0.001 && drawn < TICK_LIMIT {
        let t = (value - min) / (max - min);
        let x = track.left() + inset + t * span;
        let turns = value / TICK_MAJOR;
        let major = (turns - turns.round()).abs() < 0.001;
        let height = if major { TICK_MAJOR_HEIGHT } else { TICK_MINOR_HEIGHT };
        painter.vline(x, egui::Rangef::new(ruler.top(), ruler.top() + height), stroke);
        drawn += 1;
        value += TICK_MINOR;
    }
}

/// An 8 px accent dot with "On" in 11 px mono, right-aligned in the rect.
#[allow(dead_code)] // used by the font rows in the next task
pub fn status_dot(painter: &egui::Painter, rect: egui::Rect) {
    let galley = painter.layout_no_wrap("On".to_owned(), FontId::new(size::META, theme::mono()), color::INK_MUTED);
    let text_pos = egui::pos2(rect.right() - galley.size().x, rect.center().y - galley.size().y / 2.0);
    let dot = egui::pos2(text_pos.x - space::XS - 4.0, rect.center().y);
    painter.circle_filled(dot, 4.0, color::COBALT);
    painter.galley(text_pos, galley, color::INK_MUTED);
}
