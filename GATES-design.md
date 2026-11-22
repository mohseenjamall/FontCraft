# Gates: App design system (Engraver's Light)

OWNS: src/theme.rs, src/widgets.rs, src/app.rs, src/main.rs, assets/fonts/**, scripts/make-ui-fonts.py, GATES-design.md

Scope: docs/superpowers/plans/2026-11-22-app-design-system.md (spec: docs/superpowers/specs/2026-11-22-app-design-system-design.md). One token file, two bundled UI fonts, a few widgets; no layout change.

Budget (relative to the v1.2.0 baseline measured on 2026-11-22, see shots/design-bench-before.json): ready <= 2.2 s, working set <= 110 MB, binary <= 7.35 MiB.

- [x] G1: whole suite passes, including the token contrast test and the Arabic fallback test
  CHECK: cargo test
  EXPECT: test result: ok. 77 passed
  EVIDENCE: 77 passed; 0 failed; 4 ignored.

- [x] G2: release build has no warnings and no C runtime DLL imports
  CHECK: node scripts/build-no-warnings.mjs && node scripts/check-imports.mjs target/release/fontcraft.exe
  EXPECT: 0 warnings
  EVIDENCE: build clean; imports check passed.

- [x] G3: no inline colours or old violet outside theme.rs
  CHECK: rg -n "from_rgb|from_gray|5200EA|MAIN_BG|TEXT_MAIN|TEXT_LIGHT|ACTION|HEADER_BG|SIDE_BG|LIGHT_GRAY|Color32::GRAY" src --glob '!theme.rs'
  EXPECT: only support.rs from_rgba_unmultiplied (image decoding)
  EVIDENCE: one match, src/support.rs:21, not a colour.

- [x] G4: speed budget holds (3 runs, bench.ps1, shots/design-bench-after.json)
  EVIDENCE: before: ready 2.01 s, working set 107 MB, binary 7,391,744 B. After: idle 1.0 to 1.1 s (median 1.05 s), fonts listed and drawn 1.95 to 1.98 s, working set 103 MB (median), private 106 to 109 MB, binary 7,505,408 B (+114 KB). All inside the budget.

- [x] G5: screenshots show the new style and user fonts still preview in their own face
  EVIDENCE: shots/design-after-main.png, design-after-details.png, design-after-arabic.png checked by eye against the design-before-* shots: cobalt accent, no violet, Arabic text renders, "no glyphs" rows still dimmed.

- [ ] G6: window at 720x480 shows no clipping or overlap
  EVIDENCE: not verified (needs a human; the screenshot mode has no size flag).

- [ ] G7: keyboard focus is visible on dark and light surfaces; hover, selected and focused rows look different
  EVIDENCE: not verified (needs a human to Tab through the app).

- [ ] G8: a locked system font's text is readable
  EVIDENCE: partly checked in design-after-details.png (plain label in INK_DIM, contrast pair in the unit test); a human should confirm on a real locked font.
