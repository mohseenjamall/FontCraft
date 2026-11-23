# Gates: Grid view

OWNS: src/grid.rs, src/samples.rs, src/store.rs, src/widgets.rs, src/app.rs, src/main.rs, GATES-grid.md

Scope: docs/superpowers/plans/2026-11-22-grid-view.md (spec: docs/superpowers/specs/2026-11-22-grid-view-design.md).

Budget (design-system gates): ready <= 2.2 s, working set <= 110 MB, binary <= 7.35 MiB.

- [x] G1: whole suite passes (grid helpers, card glyphs, visible rows, view roundtrip)
  CHECK: cargo test
  EXPECT: test result: ok. 84 passed
  EVIDENCE: 84 passed; 0 failed; 4 ignored.

- [x] G2: release build has no warnings, no C runtime DLL imports
  CHECK: node scripts/build-no-warnings.mjs && node scripts/check-imports.mjs target/release/fontcraft.exe
  EVIDENCE: "release build ok, 0 warnings"; "imports ok: no C runtime DLL".

- [x] G3: default (list) view is as fast as before the grid (bench.ps1, shots/grid-bench-after.json)
  EVIDENCE: idle 0.86 to 0.9 s, fonts listed and drawn 2.01 to 2.03 s (limit 2.2), working set 102 to 104 MB (limit 110), binary 7,547,392 B = 7.20 MiB (limit 7.35; +42 KB over the design-system build).

- [ ] G4: grid view working set stays inside the budget
  EVIDENCE: NOT MET as measured once: `fontcraft.exe --view grid`, 25 s after start, working set 112 MB (limit 110), 18 threads, 1.19 s CPU. Single sample, not the 3-run bench; the list view is 104 MB, so the grid adds about 8 MB (more fonts loaded because six cards fit where one row fits).

- [x] G5: cards look right: aligned columns, ornament fonts drawn not blank, names inside the card, no overlap
  EVIDENCE: shots/grid-t2.png, grid-t2-arabic.png, grid-t3-grid.png checked by eye.

- [x] G6: the switch works and the header fits; preview text and size are dimmed in grid mode
  EVIDENCE: shots/grid-t3-list.png and grid-t3-grid.png. Choice is saved as view.txt and read back (unit test `view_roundtrips_through_a_file`); a missing or garbage file means list (same test). No stray view.txt left in %APPDATA%\FontCraft.

- [ ] G7: narrow window, long scrolls, Arabic family names, selection kept when switching views
  EVIDENCE: not verified (needs a human). Column count is unit tested for width 0, NaN, infinity and negative.
