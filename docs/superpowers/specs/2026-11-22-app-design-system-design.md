# FontCraft app: design system and restyle

Date: 2026-11-22. Status: draft for approval. Direction chosen by the owner: **A, Engraver's Light**.

## Intent and why

FontCraft looks unprofessional next to its own website: three unrelated surface colours, a violet accent that is not the brand, egui's default type, an uppercase `FONTS` label, a heavy violet `ON` pill on every row, and colour values typed inline in `app.rs`. The owner wants a professional, consistent look for the whole app, built as a real design system, **without giving up the speed and lightness that are the product's selling point** (EVALUATION.md: ~1.5 s ready, ~104 MB, 6.3 MB binary vs FontBase's 6.2 s, 784 MB, 438 MB).

The view-modes feature (grid of font cards, "Feature 2") is planned after this and builds on it.

## Character (from the site's `.impeccable.md`)

Calm, exact, a little warm. An engraver's workshop, not a SaaS dashboard. Scene: a designer compares typefaces on a bright monitor in a studio by day; type is judged on a light, paper-like surface, so the content canvas stays light and the chrome around it stays dark and quiet.

## Goals

1. One token file is the only place colours, spacing, radii, type sizes and fonts are defined.
2. Every existing screen (header, sidebar, font list, details panel, update banner, support card, notices) uses the tokens and shared widget helpers.
3. Brand match: cobalt accent (site `--cobalt #1a4ac4`, `--accent #6a85ff`), no violet left.
4. Every interactive component has default, hover, focus, active and disabled states that read apart.
5. Speed budget holds (see Constraints).

## Non-goals

- No new view modes or layout changes (list stays the only view; grid is Feature 2).
- No dark/light switch now. Tokens are grouped so a second palette can be added later.
- No website changes, no renaming, no new features, no new crates, no animation.

## Anti-goals (failure even if it "works")

- Slower startup, more memory or a bigger binary than the budget.
- A generic dark-sidebar template look: glow, gradients, glass, side-stripe borders, uppercase tracked labels, a second accent.
- Any violet (`#5200EA`) or inline `Color32::from_rgb` left in UI code.

## Constraints (the speed budget, checked by `bench.ps1`)

| Measure | Today | Budget |
|---|---|---|
| Ready and quiet, median of 3 | 1.46 s | <= 1.6 s |
| Memory working set | 104 MB | <= 110 MB |
| Windows release binary | 6.3 MB | <= 6.8 MB |

Rules that keep it there: theme applied **once** at startup (`ctx.set_style`/`set_fonts`), never per frame; no blur, shadows beyond a hairline, or per-frame allocation added to row painting; no new dependency in `Cargo.toml`; icons drawn with the painter, not an icon font.

## Approaches considered

1. **Tokens module plus small widget helpers, applied in place (chosen).** `theme.rs` holds tokens and a one-time `apply(ctx)`; `ui.rs` holds a handful of widget helpers (primary/ghost button, status dot, section label). Full control, zero dependencies, nothing to maintain but our own code.
2. **Global `Style`/`Visuals` only.** Cheapest, but egui's built-in widgets cannot express our selection row, status dot or ruler slider, so colours would stay inline. Rejected: it recreates today's problem.
3. **A third-party egui theme crate.** Adds a dependency and binary size and gives someone else's look, not the cobalt/engraver one. Rejected.

## The system

### Colour (starting values, verified by a contrast test; tinted toward the cobalt hue)

Two surfaces, three text roles each, one accent.

| Role | Chrome (header, sidebar) | Canvas (list, details) |
|---|---|---|
| Background | `#101216` header, `#15171B` sidebar | `#F7F5F0` warm paper |
| Raised / selected | `#1F2330` | `#E4EAFB` selection, `#FFFFFF` cards |
| Hairline | `#2A2D33` | `#E3DFD6` |
| Text primary | `#E8EAEE` | `#1B1C1F` |
| Text secondary | `#A3A8B1` | `#5B5F66` |
| Text muted | `#7B818B` | `#6F737A` |
| Accent | `#6A85FF` (on dark) | `#1A4AC4` (on paper) |

Semantic: danger `#B42318`, success `#2F7D4F`, used only for meaning. Primary button is cobalt `#1A4AC4` with white text. The accent marks the primary action, the selection and the "on" state, nothing else.

### Space, radius, size

- Space scale on a 4 px grid: 4, 8, 12, 16, 24, 32. Sidebar rows 32 high with 2 between; header 52; panel padding 16; page padding 24.
- Radius: controls 8, cards 12, panels square. Inner radius = outer minus padding (min 4).
- Type scale (fixed, not fluid): 11 meta and counts, 12 secondary, 13 body, 15 section title, 20 detail title. Weights 400 and 500 only. Sentence case everywhere (`FONTS` becomes `Fonts`).

### Fonts

One UI family (Archivo, static Regular and Medium, Latin subset) plus IBM Plex Mono Regular (counts, sizes, meta), both from the site's families and licensed OFL. Bundled with `include_bytes!`, registered once. egui's default fonts stay as the fallback for glyphs the subsets lack (Arabic UI text, symbols). Preview text keeps using the user's own fonts through `FontLoader`, which this work must not disturb.

### Components

Primary button, ghost button, sidebar item, search field, slider, status dot (replaces the `ON` pill: an 8 px accent dot plus "On" in 11 px mono), font row, details header, update banner, notice, empty state ("No fonts match", "Looking for fonts…"). The font card for Feature 2 is added later from the same tokens.

### Signature

A thin ruler (tick marks, echoing the site's Ruler component) along the size slider track. One detail, used only there.

### Moment

The font preview text is the only rich thing on screen. Chrome stays quiet around it.

## What is left to the builder

- Exact egui `Style` wiring (`Visuals`, widget states, `spacing`, `text_styles`) so hover/active/focus use the tokens.
- How to draw the ruler ticks and status dot with `Painter` cheaply.
- Font subsetting tool and the exact glyph ranges (Basic Latin plus Latin-1 and common punctuation).
- Replacing the selected-row fill (`#ECE6FD`) and hover grey in `row()`.

## Risks to settle early

1. Adding fonts could move binary size or startup more than expected; measure in the first task before restyling everything.
2. Registering a UI font must not collide with the user-font families the loader adds.
3. Muted text on paper is borderline for contrast; the contrast test decides the final hex.

## Verification

Unit test of token contrast pairs (>= 4.5:1 for text roles on their surface, 3:1 for the accent on non-text), the app's existing screenshot mode compared before and after, `bench.ps1` against the budget, and `rg` finds no `from_rgb` or `#5200EA` outside `theme.rs`.

## Calls I made (flag any you disagree with)

- Archivo plus Plex Mono bundled, instead of system fonts: consistent on all three OSes, about 0.2 MB.
- Status dot replaces the `ON` pill.
- "Buy me a coffee" becomes a ghost button so "Add folder" is the one primary action.
- Hex values above are starting points, not final.
