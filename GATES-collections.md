# Gates: Collections, script filter, sidebar groups

OWNS: src/collections.rs, src/samples.rs, src/scan.rs, src/app.rs, src/widgets.rs, src/activate.rs, src/activate_unix.rs, GATES-collections.md

Scope: docs/superpowers/plans/2026-11-23-collections-and-script-filter.md (spec: docs/superpowers/specs/2026-11-23-collections-and-script-filter-design.md, both amendments).

Budget: ready <= 2.2 s, working set <= 110 MB (list view), binary <= 7.35 MiB. The grid view's 112 MB is a known open item.

- [x] G1: whole suite passes (script mask, text scripts, family aggregation, collections, filters, drop folder)
  CHECK: cargo test
  EVIDENCE: 95 passed; 0 failed; 4 ignored.

- [x] G2: release build has no warnings and no C runtime DLL imports; docs checks pass
  CHECK: node scripts/build-no-warnings.mjs && node scripts/check-imports.mjs target/release/fontcraft.exe && node scripts/check-docs.mjs && node scripts/check-update-docs.mjs
  EVIDENCE: "release build ok, 0 warnings"; "imports ok"; "docs ok"; "update docs ok".

- [x] G3: speed budget in list view
  EVIDENCE: fonts listed and drawn 2.0 to 2.09 s (limit 2.2); idle 0.93 to 1.08 s; working set 110, 104, 104 MB in list view (3 manual runs of `--view list`, 25 s after start; limit 110); binary 7,573,504 B = 7.22 MiB (limit 7.35). Scan now does 16 glyph lookups per face plus the fvar check, with no visible cost.
  NOTE: bench.ps1 (shots/col-bench-after.json) starts the app with the owner's saved view, which is grid on this PC, so its 112 MB working set is the grid view's known figure (grid manual runs: 112, 113, 117 MB).

- [x] G4: the sidebar has Library, Collections and Folders; Turned on lists everything that is on; counts are cached
  EVIDENCE: shots/col-after-main.png, col-after-turnedon.png: Turned on 32 families (user-installed, FontCraft and FontBase fonts), Arabic 71, English 218, Variable 15, 4+ weights 12. Variable was 0 until ttf-parser's `variable-fonts` feature was enabled (its `is_variable()` needs it).

- [x] G5: Arabic text lists Arabic-capable fonts only, with a notice and a way back
  EVIDENCE: shots/col-after-text-ar.png "Showing fonts that support Arabic" + "Show all fonts"; Latin text and digit-only text behave per the unit tests (`visible_obeys_collection_text_and_search`).

- [x] G6: the sidebar scrolls and the footer stays visible at 720 x 480
  EVIDENCE: shots/col-after-720.png (footer and support card visible; Folders group scrolls).

- [ ] G7: removing an added folder in the real UI (hover control and Delete key)
  EVIDENCE: logic is unit tested (`dropping_a_folder_shifts_the_rest_and_repairs_state`); the hover control and keyboard path are NOT verified by eye. Removal is disabled while a scan runs.

- [ ] G8: typing Arabic live, Tab focus on the new rows, collection counts on the owner's real library
  EVIDENCE: not verified (needs a human).
