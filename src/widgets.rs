//! Shared chrome widgets: buttons, a section label, the ruler slider and the switch.
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

/// The ghost button for the light canvas (details panel); muted text when the ui is disabled.
pub fn paper_button(ui: &mut egui::Ui, label: &str) -> Response {
    let text = if ui.is_enabled() { color::INK } else { color::INK_MUTED };
    button(ui, label, Color32::TRANSPARENT, color::CANVAS_HOVER, color::SELECTED, Stroke::new(1.0, color::CANVAS_LINE), text)
}

/// Small heading above a group on the light canvas (the waterfall list).
pub fn paper_section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).font(FontId::new(size::BODY, theme::medium())).color(color::INK_DIM));
}

/// Shared drawing for the button kinds, so all keep the same metrics.
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

/// A two-or-more segment switch, 28 px high with a hairline track: the chosen segment sits on
/// `CHROME_RAISED`, the others are dim and lift on hover. Returns the segment clicked this frame.
pub fn segmented(ui: &mut egui::Ui, labels: &[&str], selected: usize) -> Option<usize> {
    const HEIGHT: f32 = 28.0;
    let font = FontId::new(size::BODY, theme::medium());
    // Placeholder colour, so the same galley can be painted as chosen or dim.
    let galleys: Vec<std::sync::Arc<egui::Galley>> = labels
        .iter()
        .map(|label| ui.painter().layout_no_wrap((*label).to_owned(), font.clone(), Color32::PLACEHOLDER))
        .collect();
    let widths: Vec<f32> = galleys.iter().map(|g| (g.size().x + 2.0 * space::M).max(HEIGHT)).collect();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(widths.iter().sum(), HEIGHT), egui::Sense::click());
    let painter = ui.painter();
    painter.rect(rect, radius::CONTROL, Color32::TRANSPARENT, Stroke::new(1.0, color::CHROME_LINE), egui::StrokeKind::Inside);
    let pointer = response.hover_pos();
    let mut left = rect.left();
    let mut clicked = None;
    for (i, galley) in galleys.into_iter().enumerate() {
        let seg = egui::Rect::from_min_size(egui::pos2(left, rect.top()), egui::vec2(widths[i], HEIGHT));
        left += widths[i];
        let inner = seg.shrink(2.0);
        let hovered = pointer.is_some_and(|p| seg.contains(p));
        let chosen = i == selected;
        if chosen {
            painter.rect(inner, radius::CONTROL, color::CHROME_RAISED, Stroke::new(1.0, color::CHROME_LINE), egui::StrokeKind::Inside);
        } else if hovered {
            painter.rect_filled(inner, radius::CONTROL, color::CHROME_HOVER);
        }
        let text = if chosen { color::CHROME_TEXT } else { color::CHROME_TEXT_DIM };
        painter.galley(seg.center() - galley.size() / 2.0, galley, text);
        if hovered && response.clicked() {
            clicked = Some(i);
        }
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    clicked
}

/// Text that acts as a link: cobalt, underlined, pointing hand.
pub fn link(ui: &mut egui::Ui, text: &str) -> Response {
    let label = egui::Label::new(RichText::new(text).size(size::SECONDARY).color(color::COBALT).underline()).sense(egui::Sense::click());
    ui.add(label).on_hover_cursor(egui::CursorIcon::PointingHand)
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

const SWITCH_W: f32 = 34.0;
const SWITCH_H: f32 = 18.0;

/// A pill switch whose right edge is at `right_center`: cobalt with the knob right when on,
/// a hairline outline with the knob left when off. The click target is a little larger.
pub fn switch(ui: &mut egui::Ui, id: egui::Id, right_center: egui::Pos2, on: bool) -> Response {
    let rect = egui::Rect::from_min_size(
        egui::pos2(right_center.x - SWITCH_W, right_center.y - SWITCH_H / 2.0),
        egui::vec2(SWITCH_W, SWITCH_H),
    );
    let response = ui.interact(rect.expand2(egui::vec2(4.0, 5.0)), id, egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    let round = SWITCH_H / 2.0;
    let knob_x = if on { rect.right() - round } else { rect.left() + round };
    let painter = ui.painter();
    if on {
        let fill = if response.hovered() { color::COBALT_HOVER } else { color::COBALT };
        painter.rect_filled(rect, round, fill);
        painter.circle_filled(egui::pos2(knob_x, rect.center().y), round - 3.0, Color32::WHITE);
    } else {
        let fill = if response.hovered() { color::CANVAS_HOVER } else { Color32::TRANSPARENT };
        painter.rect(rect, round, fill, Stroke::new(1.0, color::INK_MUTED), egui::StrokeKind::Inside);
        painter.circle_filled(egui::pos2(knob_x, rect.center().y), round - 4.5, color::INK_MUTED);
    }
    response
}

/// "Locked" in muted text, right-aligned at `right_center`, with the reason on hover (a font
/// the system owns has no switch).
pub fn locked(ui: &mut egui::Ui, id: egui::Id, right_center: egui::Pos2, reason: &str) {
    let galley = ui.painter().layout_no_wrap("Locked".to_owned(), FontId::proportional(size::META), color::INK_MUTED);
    let rect = egui::Rect::from_min_size(egui::pos2(right_center.x - galley.size().x, right_center.y - galley.size().y / 2.0), galley.size());
    ui.painter().galley(rect.min, galley, color::INK_MUTED);
    ui.interact(rect.expand(4.0), id, egui::Sense::hover()).on_hover_text(reason);
}
