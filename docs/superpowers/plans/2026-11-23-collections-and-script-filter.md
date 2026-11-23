# Collections and Script Filter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users browse fonts by state and writing system (Turned on, Arabic, English, Variable, 4+ weights), make the preview text hide fonts that cannot draw it, restructure the sidebar into Library, Collections and Folders (scrolling, removable folders), and clear up the copy across the app.

**Architecture:** Each `Face` records which of the 16 `samples::SCRIPTS` it supports (a `u16` mask) and whether it is variable; a `Family` aggregates them. Pure helpers in `samples.rs` and a new `collections.rs` are unit tested; `FontApp` gets a `collection` filter and a text-script filter inside `visible()`; the sidebar and a notice above the list expose them. A final copy pass updates existing strings.

**Tech Stack:** Rust 2024, eframe/egui 0.37, `ttf-parser` (`Face::is_variable()`), existing `theme` and `widgets`. egui 0.37 notes: `ctx.global_style()`, `run_pass(...)`, `TextEdit::align(Align2)`; read `src/app.rs` for what is really in use.

**Spec:** `docs/superpowers/specs/2026-11-23-collections-and-script-filter-design.md` (including its two Amendments sections; the later amendments win).

## Global Constraints

- Branch `collections` cut from `grid-view`. No new dependency; colours from `theme::color`, spacing from `theme::space`; sentence case; no arrow characters; no em dashes in UI copy.
- Budget: ready <= 2.2 s, working set <= 110 MB, binary <= 7.35 MiB (list view; the grid's 112 MB is a known open item). Typing in the preview box must not scan files or load fonts; collection counts are computed when families change, not per frame.
- "Turned on" = the families for which `all_active(active, disabled, family)` is true (any source, whoever turned them on; system fonts are not on).
- "English" = the Latin test letter `A` (bit 0); "Arabic" = the letter ba (bit 4); "4+ weights" = more than 3 distinct `weight` values in the family; "Variable" = any face with an `fvar` table.
- Filter labels exactly: Turned on (Library group), Arabic, English, Variable, 4+ weights (Collections group). Sidebar groups: Library, Collections, Folders; Windows label "From FontBase" replaces "Activated by FontBase". Notice exactly: "Showing fonts that support {names}" with link "Show all fonts".
- Gates: `cargo test`, `node scripts/build-no-warnings.mjs`, `node scripts/check-docs.mjs`, `node scripts/check-update-docs.mjs`.

## Review Focus

- Text mixing scripts needs both ("Hello عبد" lists only fonts with Latin and Arabic); digits, spaces and punctuation alone never filter; a script outside the 16 never filters. Pinned by `scripts_in` and `visible` tests (Tasks 1 and 2).
- An Arabic font that also has Latin letters appears in both Arabic and English; a font without the letter ba is never in Arabic. Pinned by the family aggregation test (Task 1).
- Picking a collection clears the source and the other way round; clearing the preview text restores every font; a selected family hidden by a filter keeps its details panel open (selection is not cleared by filters). Pinned by Task 2 tests.
- Every empty combination (search, collection, text filter, still scanning) shows a sentence with a reason and a next step, never a blank list. Checked in Task 3 screenshots.
- At 720 x 480 the sidebar never pushes Add folder or the support card out of view (the middle scrolls, the footer is fixed); removing an added folder re-indexes the remaining sources, faces and selection correctly and never touches files on disk. Pinned by Task 4 tests and Task 3 screenshots.
- "Turned on" updates immediately after turning a font on or off, and matches the "On" dots in the list.
- Scan time and memory stay in budget: 16 glyph lookups per face and one `is_variable()` call. Measured in Task 6.

## File Structure

- Modify `src/samples.rs`: `LATIN`, `ARABIC`, `script_mask`, `scripts_in`, `script_names`.
- Modify `src/scan.rs`: `Face.scripts`, `Face.variable`.
- Create `src/collections.rs`: `Collection` and its labels, hints and membership test.
- Modify `src/app.rs`, `src/widgets.rs` (if a link-style widget is needed), `src/main.rs` (`mod collections;`, `--collection` flag for screenshots), `src/activate.rs` (two messages), `README.md`, `MECHANICS.md`, `scripts/check-update-docs.mjs` (banner wording).
- Create `GATES-collections.md`.

---

### Task 1: Script data on faces and families

**Files:**
- Modify: `src/samples.rs`, `src/scan.rs`, `src/app.rs` (`Family`, `group`, test helper `face()`)

**Interfaces:**
- Produces: `samples::LATIN: u16 = 1 << 0`, `samples::ARABIC: u16 = 1 << 4`; `samples::script_mask(has: impl Fn(char) -> bool) -> u16` (bit i set when `SCRIPTS[i].0` is supported); `samples::scripts_in(text: &str) -> u16` (scripts of the letters in `text` by Unicode block; digits, spaces, punctuation and symbols add nothing); `samples::script_names(mask: u16) -> String` ("Arabic and English", "Arabic, English and Hebrew", "" for 0; names ordered by bit, bit 0 is "English"); `Face { scripts: u16, variable: bool }`; `Family { scripts: u16, variable: bool, weight_count: usize }` (OR of faces, any face, distinct weights).

- [ ] **Step 1: Write the failing tests:**

```rust
// samples.rs
#[test] fn mask_has_a_bit_per_supported_script() {
    assert_eq!(script_mask(|c| c == 'A' || c == 'ب'), LATIN | ARABIC);
    assert_eq!(script_mask(|_| false), 0);
    assert_eq!(script_mask(|_| true), 0xFFFF);
}
#[test] fn text_scripts_follow_the_letters_only() {
    assert_eq!(scripts_in("Hello"), LATIN);
    assert_eq!(scripts_in("عبد الرحمن"), ARABIC);
    assert_eq!(scripts_in("Hello عبد"), LATIN | ARABIC);
    for s in ["", "   ", "123 -- !?", "😀"] { assert_eq!(scripts_in(s), 0, "{s:?}"); }
}
#[test] fn names_read_naturally() {
    assert_eq!(script_names(ARABIC), "Arabic");
    assert_eq!(script_names(LATIN | ARABIC), "Arabic and English");
    assert_eq!(script_names(0), "");
}
// app.rs tests
#[test] fn family_aggregates_scripts_variable_and_weights() {
    let mut a = face("Mixed", "Regular", 400, false); a.scripts = LATIN | ARABIC;
    let mut b = face("Mixed", "Bold", 700, false); b.variable = true;
    let c = face("Mixed", "Italic", 400, true);
    let fam = &group(&[a, b, c])[0];
    assert_eq!(fam.scripts, LATIN | ARABIC);
    assert!(fam.variable);
    assert_eq!(fam.weight_count, 2); // 400 and 700; the italic repeats 400
}
```

- [ ] **Step 2: Run to verify it fails.** Run: `cargo test samples` then `cargo test aggregates` Expected: FAIL, items not defined.
- [ ] **Step 3: Implement.** `scripts_in` maps a letter to a script index by block: Latin letters (A-Z, a-z, Latin-1 and Latin Extended letters), CJK ideographs (4E00-9FFF), Hiragana/Katakana (3040-30FF), Devanagari (0900-097F), Arabic (0600-06FF, 0750-077F, FB50-FDFF, FE70-FEFF), Cyrillic (0400-04FF), Greek (0370-03FF), Khmer (1780-17FF), Lao (0E80-0EFF), Thai (0E00-0E7F), Tamil (0B80-0BFF), Myanmar (1000-109F), Bengali (0980-09FF), Hebrew (0590-05FF), Armenian (0530-058F), Georgian (10A0-10FF); use `char::is_alphabetic` so digits and symbols add nothing, and keep the index order of `SCRIPTS`. In `scan.rs` set `scripts: samples::script_mask(...)` (same glyph closure as `script_of`) and `variable: face.is_variable()`. In `group()` compute the family fields after the existing sort and dedup. Update the test helper `face()` with `scripts: 1, variable: false`.
- [ ] **Step 4: Run to verify it passes.** Run: `cargo test` Expected: PASS (existing 84 plus the new ones), `cargo build --release` with 0 warnings.
- [ ] **Step 5: Commit** `git switch -c collections grid-view` first (keep the untracked docs), then `git add src docs/superpowers && git commit -m "feat: record scripts, variable and weight count per font"`.

---

### Task 2: Collections and filter rules

**Files:**
- Create: `src/collections.rs`; Modify: `src/app.rs`, `src/main.rs`

**Interfaces:**
- Consumes: Task 1 `Family` fields and `scripts_in`.
- Produces: `collections::Collection { TurnedOn, Arabic, English, Variable, ManyWeights }` with `ALL: [Collection; 5]` (this order; `TurnedOn` is drawn in the Library group, the rest in Collections), `label(self) -> &'static str` ("Turned on", "Arabic", "English", "Variable", "4+ weights"), `hint(self) -> &'static str`, `contains(self, fam: &Family, on: bool) -> bool` (`on` is the caller's `all_active` result; only `TurnedOn` uses it); `FontApp::set_collection(&mut self, c: Option<Collection>)` (clears `source`), `FontApp::set_source(&mut self, s: Option<usize>)` (clears `collection`), `FontApp::collection_counts(&self) -> [usize; 5]` (cached field refreshed in `receive_faces`, `add_folder`, `toggle` and when a folder is removed), `FontApp::text_need(&self) -> u16` (`scripts_in(&self.text)` in list view, 0 in grid view); `visible()` also applies the collection and the text filter (`fam.scripts & need == need` when `need != 0`).

- [ ] **Step 1: Write the failing tests** in `collections.rs` and next to the `FontApp::for_test` tests:

```rust
#[test] fn collection_membership() {
    // families built with scripts/variable/weight_count set explicitly
    assert!(Collection::Arabic.contains(&fam(samples::ARABIC | samples::LATIN, false, 1), false));
    assert!(Collection::TurnedOn.contains(&fam(0, false, 1), true));
    assert!(!Collection::TurnedOn.contains(&fam(0, false, 1), false));
    assert!(!Collection::Arabic.contains(&fam(samples::LATIN, false, 1)));
    assert!(Collection::English.contains(&fam(samples::LATIN, false, 1)));
    assert!(Collection::Variable.contains(&fam(0, true, 1)));
    assert!(!Collection::ManyWeights.contains(&fam(0, false, 3)));
    assert!(Collection::ManyWeights.contains(&fam(0, false, 4)));
}
#[test] fn visible_obeys_collection_text_and_search() { /* app with Latin-only, Arabic+Latin, Arabic-only families:
    text "عبد" -> the two Arabic ones; text "Hello" -> the two Latin ones; text "Hello عبد" -> only Arabic+Latin;
    text "123" -> all three; text cleared -> all three; collection Arabic -> two; search "mix" on top narrows further;
    grid view ignores the text filter */ }
#[test] fn collection_and_source_exclude_each_other() { /* set_source(Some(1)) after set_collection(Some(Arabic)) clears the collection and vice versa */ }
#[test] fn turned_on_follows_toggle() { /* turn a font on with the test helpers: collection_counts()[0] and visible() under TurnedOn include it; turn it off: both drop it */ }
#[test] fn filters_do_not_clear_the_selection() { /* select a family, type text that hides it: selected unchanged */ }
```

- [ ] **Step 2: Run to verify it fails.** Run: `cargo test collections` and `cargo test visible_obeys` Expected: FAIL.
- [ ] **Step 3: Implement** the items above. Hints: Arabic "Fonts that include Arabic letters", English "Fonts that include Latin letters", Variable "Fonts with adjustable axes such as weight or width", ManyWeights "Families with more than 3 different weights". Add `--collection arabic|english|variable|weights` to `main.rs` for screenshots.
- [ ] **Step 4: Run all tests.** Run: `cargo test` and `cargo build --release` Expected: PASS, 0 warnings.
- [ ] **Step 5: Commit** `git add src && git commit -m "feat: collection filter and preview-text script filter"`.

---

### Task 3: Sidebar groups, notice, empty states and grid header

**Files:**
- Modify: `src/app.rs` (`sidebar`, `list`, `grid`, `header`), possibly `src/widgets.rs` for a link-style button and a hover-visible remove control

**Interfaces:**
- Consumes: Task 2 collection API; `widgets::section_label`, `source_row`.

- [ ] **Step 1: Sidebar groups.** Three groups, each with `widgets::section_label` ("Library", "Collections", "Folders") and `theme::space::M` above the second and third: Library = All fonts and `TurnedOn`; Collections = Arabic, English, Variable, 4+ weights; Folders = the source rows (Windows label "From FontBase") and the Add folder button. Filter rows use `source_row(label, count, selected)` plus `on_hover_text(hint)`; clicking the selected one clears it; all rows go through `set_collection`/`set_source` so only one filter is active.
- [ ] **Step 2: Scroll and footer.** Wrap the three groups in `egui::ScrollArea::vertical()` (no visible scrollbar until needed); keep the bottom-up footer outside it: "{families} families · {styles} styles" in mono muted, then the support card. At 720 x 480 the footer must stay visible.
- [ ] **Step 3: Notice.** When `text_need() != 0`, draw above the list one line in `color::INK_DIM` 12 px: "Showing fonts that support {script_names}" and a link-style "Show all fonts" (cobalt, underline on hover) that clears `self.text`. Nothing is drawn when the filter is off.
- [ ] **Step 4: Empty states** in `list()` and `grid()`, first match wins: still scanning and nothing yet "Scanning your fonts…"; text filter active "No fonts support {names}. Show all fonts to see everything."; Turned on chosen "Nothing is turned on yet. Turn a font on from its details panel."; another collection chosen "No {label} fonts here. Try All fonts." except Variable: "No variable fonts found. Fonts with adjustable weight or width appear here."; a search typed "No fonts match "{search}". Clear the search or pick another source."; otherwise "No fonts match."
- [ ] **Step 5: Grid header.** In grid view skip drawing the preview box and the size controls (no dimming, no "Used in the list view" tooltips); keep the search box and the switch; recompute the header width math so nothing overflows at 720 px.
- [ ] **Step 6: Look.** Run: `cargo run -- --screenshot shots/col-t3-main.png`, then with `--collection turnedon`, `--collection arabic`, `--text "عبد الرحمن"`, `--text "Hello"`, `--collection variable`, `--view grid`, `--text "zzzzzz" --size 720x480`, and the whole window at `--size 720x480`. Add `turnedon` to the `--collection` values. Open the PNGs: three labelled groups with counts, the sidebar scrolls and the footer stays visible at 720 x 480, the notice, readable empty states, no overlap, no preview box in the grid.
- [ ] **Step 7: Run tests, commit** `git add src && git commit -m "feat: sidebar groups with collections, notice and clearer empty states"`.

---

### Task 4: Remove an added folder

**Files:**
- Modify: `src/app.rs` (`FontApp::remove_folder`, sidebar folder rows), `src/store.rs` only if `save_folders` needs a new call site

**Interfaces:**
- Produces: `FontApp::remove_folder(&mut self, source: usize)` for indexes `>= fixed_sources` only (built-in sources are never removable).

- [ ] **Step 1: Write the failing tests** with `FontApp::for_test`: build an app with fixed sources, two added folders (sources 3 and 4) and faces in each. `remove_folder(3)`: the folder entry is gone, its faces are gone, faces from source 4 now have `source == 3`, `sources.len()` dropped by one, `source` and the collection reset to None if they pointed at the removed folder (or the index shifts if they pointed at a later one), and `selected` is cleared only if its family has no faces left. `remove_folder(1)` (built-in) does nothing. Files on disk are untouched (the test uses a temp dir and asserts the file still exists).
- [ ] **Step 2: Run to verify it fails.** Run: `cargo test remove_folder` Expected: FAIL.
- [ ] **Step 3: Implement.** Remove the source, drop its faces, decrement `source` of faces above it, regroup families, refresh counts, save the folder list (`store::save_folders`), and stop an in-flight scan from re-adding faces for the removed index (ignore results whose source index no longer exists). In the sidebar, added-folder rows show a small remove control on hover (tooltip "Remove folder") that calls `remove_folder`; keyboard: the row is reachable and Delete removes it.
- [ ] **Step 4: Run tests and look.** Run: `cargo test` and `cargo run -- --screenshot shots/col-t4.png --size 720x480`.
- [ ] **Step 5: Commit** `git add src && git commit -m "feat: remove an added folder from the sidebar"`.

---

### Task 5: Copy pass on existing strings

**Files:**
- Modify: `src/app.rs`, `src/activate.rs`, `README.md`, `MECHANICS.md`, `scripts/check-update-docs.mjs`, and every test that asserts a changed string (find them with `rg` before editing)

- [ ] **Step 1: Find assertions** on each old string: `rg -n "Update required|Release page|Activated by FontBase|its settings are saved|also holds|couldn't load this font|file not found|Looking for fonts" src scripts README.md MECHANICS.md docs`.
- [ ] **Step 2: Replace** exactly: banner text "FontCraft {version} is available." (buttons "Update now" or "Download update" stay; "Release page" becomes "What's new"; the banner stays non-dismissable); "Activated by FontBase" becomes "Turned on by FontBase" (Windows label only); "Off: its settings are saved" becomes "Off. Turn on restores it as it was."; "This file also holds: X (they change together)" becomes "Also in this file: X. They turn on and off together."; the Arabic sentence in the details panel becomes "FontBase may still show this font as on until you open it there."; Windows errors in `activate.rs`: "{name}: Windows couldn't load this font" becomes "Windows couldn't load {name}. The file may be damaged or blocked; try another copy." and "{name}: file not found" becomes "{name} was moved or deleted. Rescan or remove its folder." (`{name}` is the file name, not the full path). Update the docs and check script to the new banner wording.
- [ ] **Step 3: Run the gates.** Run: `cargo test`, `node scripts/check-docs.mjs`, `node scripts/check-update-docs.mjs` Expected: all pass.
- [ ] **Step 4: Commit** `git add -A src README.md MECHANICS.md scripts && git commit -m "feat: clearer copy for the banner, details panel and font errors"` (do not add `PROJECT.md`).

---

### Task 6: Budget and evidence

**Files:**
- Create: `GATES-collections.md`, `shots/col-after-*.png`, `shots/col-bench-after.json`

- [ ] **Step 1: Gates.** Run `cargo test`, `node scripts/build-no-warnings.mjs`, `node scripts/check-imports.mjs target/release/fontcraft.exe`. Expected: pass, 0 warnings. Close any running FontCraft first (it locks the release exe).
- [ ] **Step 2: Budget.** Run `powershell -ExecutionPolicy Bypass -File bench.ps1 -Runs 3` (it stops FontBase and restarts it), copy `bench-results.json` to `shots/col-bench-after.json`, then `git checkout bench-results.json`. Expected: ready <= 2.2 s, working set <= 110 MB, binary <= 7.35 MiB.
- [ ] **Step 3: Evidence.** Take the after screenshots (sidebar with counts, Arabic text filter, Latin text filter, Variable, 720 px) and write `GATES-collections.md` in the format of `GATES-design.md`: one gate per budget line and Review Focus item; mark what only a human can check (typing Arabic live, Tab focus, collection counts on the owner's real library) as not verified.
- [ ] **Step 4: Commit** `git add -f GATES-collections.md shots/col-after-*.png shots/col-bench-after.json && git commit -m "docs: collections gates and measured budget"`.

---

## Self-Review

- **Spec coverage:** script data and mask (T1), collections including Turned on and filter rules (T2), sidebar groups, scroll, footer, notice, empty states and grid header (T3), remove folder (T4), copy pass (T5), budget (T6). Out of scope per the amendments: dismissable banner, "Turned on" collection, deselect behaviour, SmartScreen note.
- **Type consistency:** `scripts: u16`, `variable: bool`, `weight_count: usize`, `Collection::{label, hint, contains(fam, on)}`, `set_collection`/`set_source`, `text_need` are named the same across tasks.
- **Review Focus:** every item has a test (T1, T2) or a screenshot or bench check (T3, T5).
- **Not verified:** exact egui 0.37 calls for the link-style text and tooltips come from existing code, not compiled; the Unicode block ranges are standard but not tested against real fonts beyond the unit tests.
