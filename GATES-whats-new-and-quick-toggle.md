# Gates: What's new banner and quick toggle

Measured 2026-11-24 on branch `quick-toggle`.

| Gate | Result |
|---|---|
| `cargo test` | 106 passed, 4 ignored (Windows registry tests) |
| `node scripts/build-no-warnings.mjs` | release build ok, 0 warnings |
| `node scripts/check-imports.mjs` | no C runtime DLL |
| `node scripts/check-whats-new.mjs` | ok; against a copy with version 9.9.9 it exits 1 naming the missing section |
| `check-docs.mjs`, `check-update-docs.mjs` | ok |
| Binary size | 7.25 MiB (limit 7.35) |
| Working set, list view, 25 s after start, 3 runs | 107.2, 103.7, 105.6 MB (limit 110) |
| Banner above the list, grid, 720x480, under the update bar | `shots/wn-after-*.png`: nothing overflows, update bar stays on top |
| Quick switch in list and grid, bulk bar | `shots/quick-list.png`, `shots/quick-grid.png` |

## Not verified (needs a person)
- A real update from the previous version shows the banner exactly once; Full release notes opens the page.
- Clicking a switch toggles the font and does not open the details panel; the confirmation above 50 families.
- Startup time (`bench.ps1`) was not re-run.
