//! egui shapes Arabic and Hebrew correctly but draws the text cursor as if the line ran left to right,
//! so new letters appeared on the wrong side of it. This draws the cursor where RTL text really grows.
use eframe::egui::{self, Align2, Stroke, TextEdit, Ui, pos2};

fn is_rtl_char(c: char) -> bool {
    matches!(c, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}')
}

/// True when every letter is Arabic or Hebrew (spaces, digits and punctuation don't count).
/// shortcut: mixed Arabic/English lines keep egui's default cursor; real bidi editing needs a text engine with it.
pub fn is_rtl(s: &str) -> bool {
    let mut letters = s.chars().filter(|c| c.is_alphabetic()).peekable();
    letters.peek().is_some() && letters.all(is_rtl_char)
}

/// Cursor x inside an RTL line, from each glyph's (left edge, advance) in typing order.
/// The cursor before letter `i` sits at that letter's right edge; at the end it sits at the last letter's left edge.
fn caret_x(glyphs: &[(f32, f32)], index: usize) -> f32 {
    match glyphs.get(index) {
        Some((x, w)) => x + w,
        None => glyphs.last().map_or(0.0, |(x, _)| *x),
    }
}

/// A one-line text box; right-aligned with a correct cursor while the person types Arabic or Hebrew.
pub fn edit(ui: &mut Ui, text: &mut String, hint: &str, width: f32) -> egui::Response {
    let rtl = is_rtl(text);
    let field = TextEdit::singleline(text).hint_text(hint).desired_width(width);
    if !rtl {
        return ui.add(field);
    }
    let caret = ui.visuals().text_cursor.stroke;
    ui.scope(|ui| {
        ui.visuals_mut().text_cursor.stroke = Stroke::NONE;
        let out = field.align(Align2::RIGHT_CENTER).show(ui);
        if out.response.has_focus()
            && let (Some(range), Some(row)) = (out.cursor_range, out.galley.rows.first())
        {
            let glyphs: Vec<(f32, f32)> = row.glyphs.iter().map(|g| (g.pos.x, g.advance_width)).collect();
            let x = out.galley_pos.x + row.pos.x + caret_x(&glyphs, range.primary.index.into());
            let y = out.galley_pos.y + row.pos.y;
            ui.painter().line_segment([pos2(x, y + row.min_y()), pos2(x, y + row.max_y())], caret);
        }
        out.response.response
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pure_arabic_and_hebrew() {
        assert!(is_rtl("مرحبا بالعالم"));
        assert!(is_rtl("שלום 123"));
        assert!(is_rtl("خط-تجربة"));
    }

    #[test]
    fn mixed_empty_and_latin_are_not_rtl() {
        assert!(!is_rtl(""));
        assert!(!is_rtl("   12 !"));
        assert!(!is_rtl("Hello"));
        assert!(!is_rtl("مرحبا hello"));
    }

    #[test]
    fn cursor_grows_leftwards_in_rtl() {
        // Three letters typed in order sit right to left: the 1st is rightmost.
        let glyphs = [(20.0, 10.0), (10.0, 10.0), (0.0, 10.0)];
        assert_eq!(caret_x(&glyphs, 0), 30.0); // before the first letter: far right
        assert_eq!(caret_x(&glyphs, 1), 20.0);
        assert_eq!(caret_x(&glyphs, 3), 0.0); // after the last letter: far left
        assert_eq!(caret_x(&[], 0), 0.0);
    }
}
