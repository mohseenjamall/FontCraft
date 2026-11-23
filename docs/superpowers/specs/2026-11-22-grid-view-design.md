# FontCraft grid view

Date: 2026-11-22. Status: approved by the owner in chat (design summary and the "constant Aa" call).

## Intent and why

People using FontCraft asked for a Font Book / FontBase style grid of font cards next to the current list, with a switch between the two. The list stays the default. The grid shows many families at a glance, one glyph pair per card, in the app's new Engraver's Light style (not a copy of FontBase).

## Goals

- A `List | Grid` switch in the header; the choice is remembered between runs.
- Grid of square cards: a large glyph pair in the font itself ("Aa" for Latin, one fitting letter per other script, e.g. the Arabic letter ain), "N styles" inside the card in muted small text, a quiet "On" dot when the family is on, the family name under the card (two lines at most).
- Click selects the family and opens the same details panel as the list; selected card uses the existing selected fill and hairline.
- Same speed profile as the list: only the cards on screen are drawn, and only their fonts load (through `FontLoader`).

## Non-goals

- No Repertoire (all glyphs) view; no change to the list view; no new dependency; no animation.

## Calls (owner approved)

- The card glyph is constant. The preview text box and the size slider only drive the list; in grid mode they are disabled with a tooltip "Used in the list view".
- Card 136 px square, 16 px gap, name area 34 px; column count follows the free width (at least 1).
- View choice is saved as `view.txt` ("list" or "grid") in the same app-data folder as `folders.txt`; a missing or unknown value means list.

## Anti-goals

- A slower start, more memory, or all families laid out every frame.
- Blank or tofu cards for fonts without Latin letters or with only ornaments.

## Constraints

Speed budget as in the design-system gates (`GATES-design.md`): idle 1.05 s, 103 MB, binary 7,505,408 B; limits ready <= 2.2 s, working set <= 110 MB, binary <= 7.35 MiB. All colours from `theme::color`, spacing from `theme::space`, sentence case.

## Approaches

1. A second drawing path in the same `FontApp` that reuses `FontLoader`, `pick_font`, selection and details (chosen): cheapest, one source of truth for state.
2. A separate grid component with its own state: more code, duplicated selection logic. Rejected.
3. Draw the grid with egui's `Grid`/wrapping layouts: lays out every card each frame, loses virtualization. Rejected.
