//! Geometry and view choice for the grid of font cards.
use crate::theme::space;

/// Which way the families are shown; the list is the default.
#[allow(dead_code)] // drawn by the grid view
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    List,
    Grid,
}

impl View {
    /// "grid" (any case, trimmed) is the grid; anything else is the list.
    #[allow(dead_code)]
    pub fn parse(s: &str) -> View {
        if s.trim().eq_ignore_ascii_case("grid") { View::Grid } else { View::List }
    }

    /// The stored form, also what `view.txt` holds.
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            View::List => "list",
            View::Grid => "grid",
        }
    }
}

/// Side of a square card.
#[allow(dead_code)]
pub const CARD_SIZE: f32 = 136.0;
/// Height reserved under the card for the family name.
#[allow(dead_code)]
pub const NAME_H: f32 = 34.0;

/// Gap between cards, and the same gap above the name area.
const GAP: f32 = space::L;

/// How many cards fit in `avail_width`, counting the gaps; never less than one.
#[allow(dead_code)]
pub fn columns(avail_width: f32) -> usize {
    if !avail_width.is_finite() || avail_width < 0.0 {
        return 1;
    }
    let fit = ((avail_width + GAP) / (CARD_SIZE + GAP)).floor();
    (fit as usize).max(1)
}

/// Rows needed for `items` at `columns` per row; zero for an empty list.
#[allow(dead_code)]
pub fn row_count(items: usize, columns: usize) -> usize {
    items.div_ceil(columns.max(1))
}

/// Height of one tile: card, then the name area below it.
#[allow(dead_code)]
pub fn row_height() -> f32 {
    CARD_SIZE + space::S + NAME_H + space::L
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_is_forgiving() {
        assert_eq!(View::parse("grid"), View::Grid);
        assert_eq!(View::parse(" Grid\n"), View::Grid);
        for s in ["", "list", "wide", "\u{0}"] {
            assert_eq!(View::parse(s), View::List);
        }
        assert_eq!(View::parse(View::Grid.as_str()), View::Grid);
    }

    #[test]
    fn columns_never_zero_and_fit() {
        for w in [0.0, 1.0, 100.0, f32::NAN, f32::INFINITY, -5.0] {
            assert!(columns(w) >= 1, "{w}");
        }
        assert_eq!(columns(136.0), 1);
        assert_eq!(columns(136.0 * 2.0 + 16.0), 2);
        assert_eq!(columns(136.0 * 2.0 + 15.0), 1);
    }

    #[test]
    fn row_count_rounds_up() {
        assert_eq!(row_count(0, 4), 0);
        assert_eq!(row_count(9, 4), 3);
        assert_eq!(row_count(8, 4), 2);
    }
}
