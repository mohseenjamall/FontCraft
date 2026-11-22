# FontCraft App Design System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the app's scattered colours and egui defaults with one token file, two bundled UI fonts and a few widget helpers, so the whole window reads as the "Engraver's Light" style, with startup, memory and binary size inside the budget.

**Architecture:** `src/theme.rs` owns tokens, a pure contrast function and a one-time `apply(ctx)` (fonts, text styles, visuals). `src/widgets.rs` owns the few shared widgets (primary and ghost button, section label, ruler slider, status dot). `app.rs` loses its colour consts and `dark_visuals()` and calls the theme and widgets. No layout change.

**Tech Stack:** Rust 2024, eframe/egui 0.37, `cargo test`, fontTools (one-off script) for the font files, `bench.ps1`, the app's `--screenshot` mode.

**Spec:** `docs/superpowers/specs/2026-11-22-app-design-system-design.md`

## Global Constraints

- Budget, from the spec: ready-and-quiet median of 3 `<= 1.6 s` (today 1.46 s); working set `<= 110 MB` (today 104); Windows release binary `<= 6.8 MB` (today 6.3).
- No new entry in `Cargo.toml`; no animation; no blur or shadow beyond a 1 px hairline; theme applied once at startup, never per frame; no per-frame allocation added to row painting.
- Sentence case only (`FONTS` becomes `Fonts`); weights 400 and 500 only; no side-stripe borders, gradients or second accent; no `#5200EA` and no `from_rgb` outside `theme.rs`.
- Space scale 4, 8, 12, 16, 24, 32 (4 px grid); radius controls 8, cards 12, panels square; header 52 high, sidebar rows 32 with 2 between, panel padding 16, page padding 24.
- Type scale 11 meta and counts, 12 secondary, 13 body, 15 section title, 20 detail title.
- Preview text keeps drawing in the user's own fonts through `FontLoader`; Arabic and Hebrew UI text must keep rendering through the lowest-priority system fallback.
- Branch: do the work on `design-system` cut from `main`.

## Review Focus

- Arabic text (the details line at `app.rs:595`, Arabic family names in row meta) still renders after Archivo is added at highest priority, i.e. the system fallback is still reached. Pinned by a test in Task 3.
- User fonts still preview in their own face and `pick_font`/`has_glyphs` still say "lacks glyphs" correctly. Pinned by the before/after screenshots in Task 6.
- Window at its minimum size (720x480): header, sidebar and details do not clip or overlap with the new 52 px header and paddings. Manual check in Task 6.
- Muted text, disabled and locked states ("System font: needs administrator") stay legible: contrast pairs in the Task 2 test include disabled and muted text, and the builder darkens a token until it passes.
- Keyboard focus is visible on both surfaces, and hover, selected and focused rows each look different. Manual Tab-through in Task 6.

## File Structure

- Create `src/theme.rs`: tokens, `contrast`, `apply`, `chrome_visuals`, text helpers.
- Create `src/widgets.rs`: shared widgets.
- Create `assets/fonts/Archivo-Regular.ttf`, `Archivo-Medium.ttf`, `PlexMono-Regular.ttf`, and `scripts/make-ui-fonts.py` that produces them.
- Modify `src/main.rs` (`mod theme; mod widgets;`), `src/app.rs` (all UI code).
- Create `GATES-design.md` (evidence, same format as the other `GATES-*.md`).

---

### Task 1: Baseline and font files

**Files:**
- Create: `scripts/make-ui-fonts.py`, `assets/fonts/Archivo-Regular.ttf`, `assets/fonts/Archivo-Medium.ttf`, `assets/fonts/PlexMono-Regular.ttf`, `shots/design-before-*.png`, `shots/design-bench-before.json`

**Interfaces:**
- Produces: three subset `.ttf` files (Basic Latin, Latin-1, general punctuation, arrows, `×`, `…`), each `<= 80 KB`, that Task 3 embeds with `include_bytes!`.

- [ ] **Step 1: Cut the branch and record the "before".** `git switch -c design-system`. Run `cargo build --release`, note the size of `target/release/fontcraft.exe`, run `powershell -ExecutionPolicy Bypass -File bench.ps1 -Runs 3` and copy `bench-results.json` to `shots/design-bench-before.json`. Take `cargo run --release -- --screenshot shots/design-before-main.png` and the same with `--select "Arial" ` (`design-before-details.png`) and `--text "مرحبا"` (`design-before-arabic.png`).
  Expected: the three PNGs exist; the binary size and median ready time are written in the commit message of Step 4.

- [ ] **Step 2: Write `scripts/make-ui-fonts.py`.** With fontTools: open the site's `site/node_modules/@fontsource-variable/archivo/files/archivo-latin-wght-normal.woff2`, instance it at `wght=400` and `wght=500` (`fontTools.varLib.instancer`), subset to the ranges above, save as TTF (`flavor=None`); convert `site/node_modules/@fontsource/ibm-plex-mono/files/ibm-plex-mono-latin-400-normal.woff2` to TTF the same way. Fonts are OFL: keep the family names.

- [ ] **Step 3: Run it and check sizes.**
  Run: `python scripts/make-ui-fonts.py && ls -l assets/fonts`
  Expected: three `.ttf` files, each `<= 80 KB`.

- [ ] **Step 4: Commit**

```bash
git add scripts/make-ui-fonts.py assets/fonts shots/design-before-*.png shots/design-bench-before.json
git commit -m "chore: record design baseline and add UI font files"
```

---

### Task 2: Tokens and the contrast test

**Files:**
- Create: `src/theme.rs`
- Modify: `src/main.rs` (add `mod theme;`)

**Interfaces:**
- Produces: `theme::color` consts (all `egui::Color32`): `HEADER` `#101216`, `SIDEBAR` `#15171B`, `CHROME_RAISED` `#1F2330`, `CHROME_HOVER` `#1A1D23`, `CHROME_LINE` `#2A2D33`, `CHROME_TEXT` `#E8EAEE`, `CHROME_TEXT_DIM` `#A3A8B1`, `CHROME_TEXT_MUTED` `#7B818B`, `ACCENT_ON_DARK` `#6A85FF`, `PAPER` `#F7F5F0`, `CARD` `#FFFFFF`, `SELECTED` `#E4EAFB`, `CANVAS_HOVER` `#EFECE5`, `CANVAS_LINE` `#E3DFD6`, `INK` `#1B1C1F`, `INK_DIM` `#5B5F66`, `INK_MUTED` `#6F737A`, `COBALT` `#1A4AC4`, `COBALT_HOVER` `#2457D6`, `COBALT_PRESSED` `#173FA8`, `DANGER` `#B42318`, `SUCCESS` `#2F7D4F`. Also `theme::space::{XS=4,S=8,M=12,L=16,XL=24,XXL=32}`, `theme::radius::{CONTROL=8,CARD=12}`, `theme::size::{META=11,SECONDARY=12,BODY=13,TITLE=15,DETAIL=20}` (`f32`), `theme::HEADER_HEIGHT=52`, `theme::ROW_HEIGHT=32`.
- Produces: `pub fn contrast(a: Color32, b: Color32) -> f32` (WCAG relative-luminance ratio).
- Hover, pressed and `CHROME_HOVER` values are this plan's calls; the hexes above are starting points, the test decides.

- [ ] **Step 1: Write the failing tests** in `theme.rs` (`#[cfg(test)]`):

```rust
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
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test theme`
Expected: FAIL, `contrast` / `color` not defined.

- [ ] **Step 3: Implement the tokens and `contrast` in `src/theme.rs`** with exactly the names above. `contrast` uses sRGB linearisation and `(L1+0.05)/(L2+0.05)`.

- [ ] **Step 4: Run, then fix tokens.**
  Run: `cargo test theme`
  Expected: PASS. If a pair fails (`INK_MUTED` on `PAPER` is expected to be borderline), darken that token until it passes and keep the new hex in the consts.

- [ ] **Step 5: Commit**

```bash
git add src/theme.rs src/main.rs
git commit -m "feat: design tokens and contrast test"
```

---

### Task 3: Fonts, text styles and visuals applied once

**Files:**
- Modify: `src/theme.rs`, `src/app.rs` (`FontApp::new`, remove `dark_visuals`, line ~257)

**Interfaces:**
- Consumes: Task 2 tokens; the three TTFs from Task 1.
- Produces: `theme::apply(ctx: &egui::Context)` (called once, in `FontApp::new`, before `add_system_fallback`); `theme::chrome_visuals() -> egui::Visuals` (replaces `dark_visuals()` for header and sidebar); `theme::medium() -> egui::FontFamily` (the `"ui-medium"` family, Archivo Medium, for fixed English labels only); `theme::mono() -> egui::FontFamily` (Plex Mono).

- [ ] **Step 1: Write the failing tests** in `theme.rs`:

```rust
#[test] fn apply_sets_the_text_scale() {
    let ctx = egui::Context::default(); apply(&ctx);
    let s = ctx.style();
    assert_eq!(s.text_styles[&egui::TextStyle::Body].size, size::BODY);
    assert_eq!(s.text_styles[&egui::TextStyle::Heading].size, size::DETAIL);
    assert_eq!(s.text_styles[&egui::TextStyle::Small].size, size::META);
}
#[test] fn arabic_still_reaches_a_fallback_font() {
    // only meaningful where the OS has an Arabic font to borrow
    if !cfg!(windows) { return; }
    let ctx = egui::Context::default(); apply(&ctx); crate::fonts::add_system_fallback(&ctx);
    let _ = ctx.run(Default::default(), |_| {}); // fonts take effect on the next frame
    let ok = ctx.fonts_mut(|f| f.has_glyphs(&egui::FontId::proportional(13.0), "مرحبا"));
    assert!(ok);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test theme::`
Expected: FAIL, `apply` not defined.

- [ ] **Step 3: Implement `apply`.** Register Archivo Regular at `FontPriority::Highest` in `Proportional`, Archivo Medium alone in `Name("ui-medium")`, Plex Mono at `Highest` in `Name("ui-mono")` (use `ctx.add_font(FontInsert::new(..., FontData::from_static(include_bytes!(..)), ..))` like `fonts.rs`, so the system fallback and the user families are untouched). Set text styles to the size scale, then `ctx.set_visuals` with `PAPER` panel fill, `INK` text, `COBALT` selection and hyperlink, `CANVAS_LINE` strokes, 8 px widget corner radius, no shadows; and `ctx.style_mut` spacing from `theme::space`. Write `chrome_visuals()` the same way from the chrome tokens, and delete `dark_visuals()` and its use. In `FontApp::new` replace `ctx.set_visuals(egui::Visuals::light())` with `crate::theme::apply(ctx)`.

- [ ] **Step 4: Run all tests and look at it**

Run: `cargo test` then `cargo run --release -- --screenshot shots/design-t3.png`
Expected: all tests PASS; the screenshot shows the new font and cobalt selection with the old layout.

- [ ] **Step 5: Commit**

```bash
git add src/theme.rs src/app.rs
git commit -m "feat: apply theme fonts, text styles and visuals once at startup"
```

---

### Task 4: Chrome (banner, header, sidebar, support card) and shared widgets

**Files:**
- Create: `src/widgets.rs`
- Modify: `src/main.rs` (`mod widgets;`), `src/app.rs` (`update_banner`, `header`, `sidebar`, `support_card`)

**Interfaces:**
- Consumes: Task 3 `chrome_visuals`, `medium`, `mono`; Task 2 tokens.
- Produces (all in `widgets`): `primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response` (cobalt, white text, hover `COBALT_HOVER`, pressed `COBALT_PRESSED`); `ghost_button(ui, label) -> Response` (1 px hairline, transparent, hover raised); `section_label(ui, text: &str)` (13 px medium, secondary colour, sentence case); `ruler_slider(ui, value: &mut f32, range: RangeInclusive<f32>, step: f64) -> Response`; `status_dot(painter: &egui::Painter, rect: egui::Rect)` (8 px accent dot with "On" in 11 px mono at the rect's right edge).

- [ ] **Step 1: Implement the widgets** in `widgets.rs`. `ruler_slider` adds the normal `egui::Slider` (suffix " px") and paints 1 px ticks under its track with the `Painter`: minor every 6 px of value, major (taller) every 30. Ticks use the chrome hairline colour; total painter calls stay under 40 per frame.

- [ ] **Step 2: Restyle the chrome in `app.rs`.** Header 52 high with `HEADER` fill and 16 px padding; the text and search inputs use `CHROME_RAISED` fill and the hairline; the size control becomes `ruler_slider`. Sidebar: `SIDEBAR` fill; `FONTS` becomes `section_label("Fonts")`; rows 32 high, 2 apart, count right-aligned in `mono()` muted, selected row = `CHROME_RAISED` fill plus 1 px hairline (no side stripe), hover `CHROME_HOVER`; `+ Add folder` becomes `primary_button` (the one primary action); "Buy me a coffee" becomes `ghost_button`; the card keeps its picture on a `CHROME_RAISED` background with the 12 px card radius. Update banner uses `COBALT` fill, white text, a white `primary`-style button and a ghost "Release page". The status text `{n} styles` stays muted in `mono()`.

- [ ] **Step 3: Build, test and look**

Run: `cargo test` then `cargo run --release -- --screenshot shots/design-t4.png`
Expected: tests PASS; screenshot shows sentence-case "Fonts", a cobalt Add folder button, ruler ticks under the slider, no violet in the header, sidebar or banner (also run once with `FONTCRAFT_LATEST=999.0.0` to see the banner).

- [ ] **Step 4: Commit**

```bash
git add src/widgets.rs src/main.rs src/app.rs
git commit -m "feat: restyle header, sidebar and banner with the design tokens"
```

---

### Task 5: Canvas (list rows, empty states, details panel)

**Files:**
- Modify: `src/app.rs` (`list`, `row`, `details`; delete the old colour consts at lines 14-19)

**Interfaces:**
- Consumes: Task 4 `widgets::{primary_button, ghost_button, section_label, status_dot}`; Task 3 text helpers.

- [ ] **Step 1: Restyle `list` and `row`.** `CentralPanel` fill `PAPER`, 24 px page padding; row meta line in `mono()` 11 px `INK_MUTED` (name, style count); selected row `SELECTED` fill with a 1 px `CANVAS_LINE` border and `radius::CONTROL`, hover `CANVAS_HOVER`; the `ON` pill is replaced by `status_dot`; preview text keeps `INK` and the user's own font. Empty states ("No fonts match.", "Looking for fonts…") centred in `INK_MUTED` 13 px. Keep the row painting allocation-free beyond what it does today.

- [ ] **Step 2: Restyle `details`.** Panel fill `CARD`, 16 px padding; title at 20 px regular (it can show an Arabic family name, so not the medium family); path in `mono()` muted; "Turn on" is `primary_button`, "Turn off" and the locked state are `ghost_button` (locked one disabled, text still `>= 4.5:1` via `INK_MUTED`); notice text uses `DANGER`; "On for all apps" uses `COBALT`; `WATERFALL` heading becomes `section_label("Waterfall")`; selected style row uses `SELECTED`.

- [ ] **Step 3: Remove the old consts and prove nothing is left**

Run: `rg "from_rgb|5200EA|HEADER_BG|SIDE_BG|MAIN_BG|TEXT_LIGHT|TEXT_MAIN|ACTION" src --glob '!theme.rs'`
Expected: no matches (the `Color32::from_gray` calls in `pick_font` and `row` are replaced with tokens too; the grep also shows none for `from_gray`).

- [ ] **Step 4: Test and look**

Run: `cargo test` then `cargo run --release -- --screenshot shots/design-t5.png --select Arial`
Expected: tests PASS; list on warm paper, details panel with cobalt Turn on, no violet anywhere.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: restyle list rows and details panel with the design tokens"
```

---

### Task 6: Budget, evidence and manual checks

**Files:**
- Create: `GATES-design.md`, `shots/design-after-*.png`, `shots/design-bench-after.json`

- [ ] **Step 1: Run the repo gates**

Run: `node scripts/build-no-warnings.mjs` then `node scripts/check-imports.mjs target/release/fontcraft.exe`
Expected: a clean build with no warnings; the imports check passes.

- [ ] **Step 2: Measure the budget**

Run: `powershell -ExecutionPolicy Bypass -File bench.ps1 -Runs 3` then copy `bench-results.json` to `shots/design-bench-after.json`, and read the binary size.
Expected: ready median `<= 1.6 s`, working set `<= 110 MB`, binary `<= 6.8 MB`. If any is over, stop and report which font file or change caused it (drop `Archivo-Medium` first) instead of loosening the budget.

- [ ] **Step 3: Take the "after" shots** with the same three commands as Task 1 (`design-after-main.png`, `design-after-details.png`, `design-after-arabic.png`) and compare each with its "before": user fonts still preview in their own face, Arabic text is not boxes.

- [ ] **Step 4: Manual checks**, each written as pass or fail in `GATES-design.md`: (a) resize the window to 720x480, nothing clips or overlaps; (b) Tab through header, sidebar and a row, the focus ring is visible on dark and light surfaces; (c) hover, selected and keyboard-focused rows look different; (d) a locked system font's disabled button text is readable.

- [ ] **Step 5: Write `GATES-design.md`** in the format of `GATES-update.md`: one gate per budget line and per manual check, with the measured numbers as evidence.

- [ ] **Step 6: Commit**

```bash
git add GATES-design.md shots/design-after-*.png shots/design-bench-after.json
git commit -m "docs: design system gates and measured budget"
```

---

## Self-Review

- **Spec coverage:** tokens file (T2), fonts and one-time apply (T3), widgets and chrome incl. ruler signature and status dot (T4), canvas and details (T5), speed budget and evidence (T1, T6), no inline colours (T5 grep). Non-goals (grid, theme switch, site, new crates) appear nowhere.
- **Type consistency:** `theme::apply`, `chrome_visuals`, `medium`, `mono`, and the five widget names are used with the same signatures in Tasks 3 to 5.
- **Review Focus:** Arabic fallback has a test (T3), contrast has a test (T2), the other three have named manual or screenshot checks (T6).
- **Not verified:** egui 0.37 method names for `add_font`/`FontData::from_static`/`Style` fields are taken from `fonts.rs` and egui conventions, not compiled; `archivo-latin-wght-normal.woff2` exists in the site's `node_modules` (checked). The muted-ink and hover hexes are starting values.
