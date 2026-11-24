# What's New Banner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show a one-time "New in {version}" banner with short notes on the first launch after an update.

**Architecture:** The notes live in `assets/whats-new.md` and are embedded with `include_str!`. A pure module `src/whats_new.rs` parses a version's bullets and decides when to show them; `store.rs` remembers the last version seen; `FontApp` holds the notes to show and draws a dismissable banner between the sidebar/details and the list. A small script and a unit test make a missing section fail the build.

**Tech Stack:** Rust 2024, eframe/egui 0.37, existing `theme`, `widgets`, `update` modules, Node script, GitHub Actions. egui 0.37 notes: read `src/app.rs` (`update_banner`) for the real panel and widget API.

**Spec:** `docs/superpowers/specs/2026-11-23-whats-new-banner-design.md`

## Global Constraints

- Branch `whats-new` cut from `collections` (it needs the current `app.rs` layout). No new dependency; no network code; colours from `theme::color`, spacing from `theme::space`; sentence case; no em dashes in notes or UI copy.
- Banner title "New in {version}"; buttons "Got it" (primary) and "Full release notes" (link). Notes: 3 to 6 bullets, one line each, in `assets/whats-new.md` as `## {version}` sections, newest first.
- Shown once per version, never on a fresh install, never on the same or an older version; the "update available" banner is untouched and stays above everything.
- Seen version file: `seen_version.txt` in the same folder as `folders.txt`.
- Budget: ready <= 2.2 s, working set <= 110 MB (list view), binary <= 7.35 MiB.
- Gates: `cargo test`, `node scripts/build-no-warnings.mjs`, `node scripts/check-docs.mjs`, `node scripts/check-update-docs.mjs`, `node scripts/check-whats-new.mjs`.

## Review Focus

- Fresh install (no seen file) shows nothing and records the version; same version and downgrades show nothing. Pinned by `on_start` tests (Task 1).
- Skipped versions (seen 1.1.0, now 1.4.0) show only 1.4.0's notes; a missing or empty section shows no banner but still records the version, no crash. Pinned by `on_start` tests.
- A seen file that is empty, whitespace or garbage ("abc") is treated like no file: nothing shown, version recorded. Pinned by `should_show` and `on_start` tests.
- The notes file has Windows line endings in this repo (CRLF) and may have blank lines or indented bullets; parsing must not depend on `\n` only. Pinned by a CRLF `parse_notes` test.
- The banner never blocks: Got it closes it, a long bullet wraps inside the width, and at 720 x 480 nothing overflows. Checked in Task 2 screenshots.

## File Structure

- Create `assets/whats-new.md`, `src/whats_new.rs`, `scripts/check-whats-new.mjs`, `GATES-whats-new.md`.
- Modify `src/store.rs` (seen version), `src/update.rs` (`release_page`), `src/app.rs` (state, banner, startup), `src/main.rs` (`mod whats_new;`, `--whats-new` flag), `.github/workflows/ci.yml` and `release.yml` (run the script), `README.md`, `MECHANICS.md`.

---

### Task 1: Parsing, show rules and remembered version

**Files:**
- Create: `src/whats_new.rs`, `assets/whats-new.md`; Modify: `src/store.rs`, `src/main.rs` (`mod whats_new;`, with a temporary `#[allow(dead_code)]` until Task 2)

**Interfaces:**
- Produces: `whats_new::NOTES: &str` (`include_str!("../assets/whats-new.md")`); `whats_new::parse_notes(markdown: &str, version: &str) -> Option<Vec<String>>` (bullets of the `## {version}` section, text after `- `, trimmed, blank lines skipped; `None` when the section is absent or has no bullets; handles `\r\n`); `whats_new::should_show(seen: Option<&str>, current: &str) -> bool` (true only when `seen` is a valid version and `current` is newer, using `update::is_newer(current, seen)`); `whats_new::Start { notes: Option<Vec<String>>, save_version: bool }` and `whats_new::on_start(seen: Option<&str>, current: &str, markdown: &str) -> Start`; `store::read_seen(path: &Path) -> Option<String>`, `store::write_seen(path: &Path, version: &str)`, `store::load_seen_version() -> Option<String>`, `store::save_seen_version(version: &str)` (file `seen_version.txt`).

- [ ] **Step 1: Write the failing tests** (in `whats_new.rs` and `store.rs`):

```rust
const MD: &str = "## 1.4.0\n- Collections\n- Grid view\n\n## 1.3.0\n- Older thing\n";
#[test] fn parses_a_versions_bullets() {
    assert_eq!(parse_notes(MD, "1.4.0"), Some(vec!["Collections".into(), "Grid view".into()]));
    assert_eq!(parse_notes(MD, "1.3.0"), Some(vec!["Older thing".into()]));
    assert_eq!(parse_notes(MD, "9.9.9"), None);
    assert_eq!(parse_notes("## 1.0.0\nno bullets here\n", "1.0.0"), None);
}
#[test] fn parsing_handles_windows_line_endings_and_indents() {
    assert_eq!(parse_notes("## 1.4.0\r\n- One\r\n  - Two  \r\n\r\n## 1.3.0\r\n- Old\r\n", "1.4.0"), Some(vec!["One".into(), "Two".into()]));
}
#[test] fn show_rules() {
    assert!(should_show(Some("1.2.0"), "1.3.0"));
    assert!(!should_show(Some("1.3.0"), "1.3.0")); // same version
    assert!(!should_show(Some("1.4.0"), "1.3.0")); // downgrade
    for seen in [None, Some(""), Some("  "), Some("abc")] { assert!(!should_show(seen, "1.3.0"), "{seen:?}"); }
}
#[test] fn on_start_decides_notes_and_saving() {
    let fresh = on_start(None, "1.4.0", MD);
    assert_eq!((fresh.notes, fresh.save_version), (None, true));
    let upgraded = on_start(Some("1.1.0"), "1.4.0", MD); // skipped versions: only 1.4.0's notes
    assert_eq!(upgraded.notes, Some(vec!["Collections".into(), "Grid view".into()]));
    assert!(upgraded.save_version);
    let no_notes = on_start(Some("1.1.0"), "2.0.0", MD); // new version without notes: nothing shown, still recorded
    assert_eq!((no_notes.notes, no_notes.save_version), (None, true));
    let same = on_start(Some("1.4.0"), "1.4.0", MD);
    assert_eq!((same.notes, same.save_version), (None, false));
    let garbage = on_start(Some("abc"), "1.4.0", MD);
    assert_eq!((garbage.notes, garbage.save_version), (None, true));
}
#[test] fn bundled_notes_cover_the_current_version() {
    assert!(parse_notes(NOTES, env!("CARGO_PKG_VERSION")).is_some_and(|n| (1..=6).contains(&n.len())));
}
// store.rs
#[test] fn seen_version_roundtrips_through_a_file() { /* temp dir: write_seen then read_seen gives the version trimmed; a missing file gives None */ }
```

- [ ] **Step 2: Run to verify it fails.** Run: `cargo test whats_new store` run as two commands (`cargo test whats_new`, `cargo test seen_version`). Expected: FAIL, items not defined.
- [ ] **Step 3: Implement** the items above. `should_show` and `on_start` treat anything `update::is_newer` cannot compare as "no usable seen version". `assets/whats-new.md`: create the `## {CARGO_PKG_VERSION}` section with 3 to 6 plain-English bullets taken from `git log` since the previous release tag; each bullet one line, sentence case, no em dashes.
- [ ] **Step 4: Run to verify it passes.** Run: `cargo test` Expected: PASS, no warnings in `cargo build`.
- [ ] **Step 5: Commit** `git switch -c whats-new collections`, then `git add src assets docs/superpowers && git commit -m "feat: whats new notes, show rules and remembered version"`.

---

### Task 2: Banner and startup wiring

**Files:**
- Modify: `src/update.rs` (`release_page`), `src/app.rs`, `src/main.rs`

**Interfaces:**
- Consumes: Task 1 items.
- Produces: `update::release_page(version: &str) -> String` (`{RELEASE_PAGES}tag/v{version}`, passes `update::trusted`); `FontApp.whats_new: Option<Vec<String>>`; `FontApp::show_whats_new(&mut self)` (forces the current version's notes, for screenshots); CLI flag `--whats-new`.

- [ ] **Step 1: Write the failing tests:** `release_page_is_trusted` in `update.rs` (`assert!(trusted(&release_page("1.3.0")))`, ends with `tag/v1.3.0`); `got_it_hides_the_banner` in `app.rs` tests (`for_test` app with `whats_new = Some(vec!["x".into()])`; calling the method that handles the Got it click leaves `whats_new == None`).
- [ ] **Step 2: Run to verify it fails.** Run: `cargo test release_page` Expected: FAIL.
- [ ] **Step 3: Implement.** In `FontApp::new`: `let start = whats_new::on_start(store::load_seen_version().as_deref(), env!("CARGO_PKG_VERSION"), whats_new::NOTES)`; keep `start.notes` in the new field; if `start.save_version` call `store::save_version`. The seen version is written at startup (shown means seen). Draw the banner in `ui()` after `self.details(ui)` and before `self.list(ui)`: a top panel filling `color::CARD` with a 1 px `color::CANVAS_LINE` border, 16 px padding; line 1 "New in {version}" in cobalt medium 15 px; bullets in `color::INK_DIM` 13 px wrapped; on the right "Got it" (`widgets::primary_button`) and "Full release notes" (`widgets::link`, opens `update::release_page(version)` through `update::open`, errors go to `self.notice`). Wire `--whats-new` in `main.rs` like `--select`. Remove the Task 1 `allow(dead_code)`.
- [ ] **Step 4: Run tests and look.** Run: `cargo test`, then `cargo run -- --screenshot shots/wn-t2.png --view list --whats-new`, and again with `--size 720x480` and `--view grid`. Expected: tests pass; the banner sits above the list between the sidebar and the right edge, bullets wrap, nothing overflows at 720 px, the update banner (run with `FONTCRAFT_LATEST=999.0.0`) stays above it.
- [ ] **Step 5: Commit** `git add src && git commit -m "feat: whats new banner after an update"`.

---

### Task 3: Release guard and docs

**Files:**
- Create: `scripts/check-whats-new.mjs`; Modify: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `README.md`, `MECHANICS.md`

- [ ] **Step 1: Write the script.** `scripts/check-whats-new.mjs` reads `version` from `Cargo.toml`, requires a `## {version}` section in `assets/whats-new.md` with 1 to 6 bullet lines (handle CRLF), prints `whats-new ok` or the problem and exits 1.
- [ ] **Step 2: Run it both ways.** Run: `node scripts/check-whats-new.mjs` Expected: `whats-new ok`. Then change the version in a scratch copy of `Cargo.toml` in a temp folder (do not edit the real one) and run the script against it with the temp path as its argument (`node scripts/check-whats-new.mjs <dir>`, default the current folder). Expected: exit code 1 and a message naming the missing section.
- [ ] **Step 3: Hook it up.** Add a step running the script before the build in the `test` job of `ci.yml` and before the build in `release.yml`. Document in `README.md` (a "Releasing" note: add the `## {version}` section to `assets/whats-new.md` and bump `Cargo.toml` together) and in `MECHANICS.md` (seen version file, once per version, fresh installs skip it).
- [ ] **Step 4: Run the doc checks.** Run: `node scripts/check-docs.mjs` and `node scripts/check-update-docs.mjs` Expected: both pass.
- [ ] **Step 5: Commit** `git add scripts .github README.md MECHANICS.md && git commit -m "ci: fail the build when the current version has no whats new notes"`.

---

### Task 4: Budget and evidence

**Files:**
- Create: `GATES-whats-new.md`, `shots/wn-after-*.png`

- [ ] **Step 1: Gates.** Run `cargo test`, `node scripts/build-no-warnings.mjs`, `node scripts/check-imports.mjs target/release/fontcraft.exe`, `node scripts/check-whats-new.mjs`. Expected: all pass, 0 warnings. Close any running FontCraft first.
- [ ] **Step 2: Budget.** Measure three runs of `fontcraft.exe --view list` (working set 25 s after start) and check the binary size. Expected: working set <= 110 MB, binary <= 7.35 MiB, startup unchanged (the banner is drawn only when notes exist; the only startup cost is one small file read).
- [ ] **Step 3: Evidence.** Take the after screenshots (banner in list, grid and 720 x 480) and write `GATES-whats-new.md` in the format of `GATES-collections.md`: one gate per budget line and Review Focus item; mark what only a human can check (a real update from the previous version showing the banner exactly once, clicking Full release notes) as not verified.
- [ ] **Step 4: Commit** `git add -f GATES-whats-new.md shots/wn-after-*.png && git commit -m "docs: whats new gates and measured budget"`.

---

## Self-Review

- **Spec coverage:** notes file, parsing and once-per-version rules (T1), banner with Got it and Full release notes, startup, screenshot flag (T2), build guard and docs (T3), budget (T4). Non-goals (fetching from GitHub, translations, pop-ups) appear nowhere.
- **Type consistency:** `parse_notes`, `should_show`, `Start { notes, save_version }`, `on_start`, `NOTES`, `read_seen`/`write_seen`/`load_seen_version`/`save_seen_version`, `release_page` and `FontApp.whats_new` are named identically across tasks.
- **Review Focus:** fresh install, skipped versions, garbage seen file and CRLF are unit tests (T1); the layout checks are screenshots (T2).
- **Not verified:** `update::trusted` and `update::is_newer` signatures were read from names in `update.rs` and `MECHANICS.md`, not compiled; the panel ordering that keeps the banner right of the sidebar assumes egui lays out later top panels in the remaining area, so check it in the Task 2 screenshots. A fresh Cargo version bump is the owner's release step, so the notes file must be updated with it.
