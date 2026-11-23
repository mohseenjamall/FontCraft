# Quick toggle Implementation Plan

> Execute task by task; steps use checkboxes.

**Goal:** One-click switch per family and a bulk bar that follows the filter.
**Architecture:** `widgets::switch` draws and senses the pill. `FontApp::row`/`card` call it and then `toggle`. `FontApp::bulk_in` gathers files of the shown families and calls `turn_on_in`/`turn_off_in` once. `bulk_bar` draws the bar and the confirmation.
**Spec:** docs/superpowers/specs/2026-11-24-quick-toggle-design.md

## Global Constraints
English UI strings, sentence case, no em dashes. List stays default. No new dependency. Budget: ready <= 2.2 s, list view <= 110 MB.

## Review Focus
- Locked families in a bulk run are skipped, never error out the batch (test).
- Families sharing one file are not counted twice (test).
- Clicking the switch does not select the row (manual screenshot).
- Changing the filter while a confirmation is pending cancels it (test).

### Task 1: Bulk logic (tests first)
Files: `src/app.rs`. Interfaces: `fn bulk_plan(&self, shown: &[usize], on: bool) -> (Vec<PathBuf>, usize /*families*/, usize /*locked*/)`; `fn bulk_in(&mut self, shown: &[usize], on: bool, key: &str, store: &Path)`; field `pending_bulk: Option<bool>`, `bulk_note: Option<String>`.
- [ ] Tests: locked skipped and counted; already-in-state skipped; shared file deduped; pending cleared by `cancel_bulk`.
- [ ] Implement.
- [ ] `cargo test`.
- [ ] Commit.

### Task 2: Switch widget and rows/cards
Files: `src/widgets.rs`, `src/app.rs`. Interfaces: `widgets::switch(ui, rect, on) -> Response`.
- [ ] Implement switch; use in `row` and `card`; locked shows "Locked" text + tooltip; update empty text.
- [ ] Screenshot list and grid (`--view`), check the click does not open details.
- [ ] Commit.

### Task 3: Bulk bar
Files: `src/app.rs`.
- [ ] `bulk_bar` in `list` and `grid`; confirmation above 50; note after run.
- [ ] Screenshots, README/MECHANICS line, gates (`cargo clippy`, tests, budget bench).
- [ ] Commit.
