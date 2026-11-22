# Download Count Badge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show the total number of FontCraft downloads (all versions, all OSes) as a small badge next to the Download button on the site.

**Architecture:** The site is a static Astro page that already calls the GitHub API from the browser. Add one pure helper module (sum + format, unit tested with `node --test`) and one extra `fetch` of `/releases?per_page=100` in the existing inline script of `index.astro`. The badge is `hidden` in the HTML and only revealed when a valid count arrives, so a failed call leaves the page unchanged.

**Tech Stack:** Astro 7 static site, plain `.mjs` helpers, `node:test`, GitHub REST API (no token).

**Spec:** No spec doc. Decisions made with the owner on 2026-11-22: (1) count = sum of GitHub release asset `download_count` now; a real "first run" ping from the app (Cloudflare Worker + KV) is a **separate later plan**, not here; (2) shown as a small badge near the Download buttons, not a stats section.

## Global Constraints

- Static site only: no backend, no new dependency, no API token (`site/package.json` unchanged).
- Repo: `mohseenjamall/FontCraft`; only count assets whose URL belongs to that repo (same trust check the page already does).
- English copy, sentence case, no uppercase tracked labels (site GATES G4). Text uses existing `--ink-dim`/`--font-mono` tokens; no new colours, no gradients.
- Page must still build to one static page: `npm run build` prints `Complete!` (site GATES G2).
- Network failure, rate limit (403), or bad JSON must leave the page exactly as it is today.

## Review Focus

- GitHub returns 403 (unauthenticated rate limit, 60/hour/IP) → badge stays hidden, no console error shown to the visitor, no broken layout.
- Total is 0 (new repo or API quirk) → badge stays hidden; "0 downloads" would look worse than nothing.
- `download_count` missing, a string, negative or `NaN` on an asset → that asset counts as 0; the sum is never `NaN`.
- Checksum and signature files (`SHA256SUMS.txt`, `*.sig`) are not real downloads → excluded from the sum.
- Response is not an array (error object, `null`) or a release has no `assets` → helper returns 0, never throws.
- Large numbers: 1,234 → `1.2K`, 12,500 → `12.5K`, 1,250,000 → `1.3M`; below 1,000 shown exactly.

## File Structure

- Create `site/src/lib/sum-downloads.mjs`: `sumDownloads`, `formatCount` (pure, no DOM).
- Create `site/scripts/sum-downloads.test.mjs`: unit tests, same style as `pick-asset.test.mjs`.
- Modify `site/src/components/GetIt.astro`: add the hidden badge under the Download button, plus its style.
- Modify `site/src/pages/index.astro`: fetch the releases list, fill and reveal the badge.

---

### Task 1: Pure count helpers

**Files:**
- Create: `site/src/lib/sum-downloads.mjs`
- Test: `site/scripts/sum-downloads.test.mjs`

**Interfaces:**
- Produces: `sumDownloads(releases: unknown): number` and `formatCount(n: number): string`, both named exports. Task 2 imports them.

- [ ] **Step 1: Write the failing tests** in `site/scripts/sum-downloads.test.mjs` (imports `{ sumDownloads, formatCount }` from `../src/lib/sum-downloads.mjs`):

```js
const rel = (...assets) => ({ assets: assets.map(([name, download_count]) => ({ name, download_count })) });

test('sums every installable asset across releases', () =>
  assert.equal(sumDownloads([rel(['FontCraft-1.2.0-windows-x64.exe', 100], ['FontCraft-1.2.0-macos.dmg', 20]), rel(['FontCraft-1.1.0-windows-x64.zip', 5])]), 125));
test('ignores checksum and signature files', () =>
  assert.equal(sumDownloads([rel(['SHA256SUMS.txt', 900], ['FontCraft-1.2.0-macos.dmg.sig', 900], ['FontCraft-1.2.0-linux-x64', 3])]), 3));
test('bad download_count counts as 0, never NaN', () =>
  assert.equal(sumDownloads([rel(['a.exe', undefined], ['b.exe', '7'], ['c.exe', -4], ['d.exe', NaN], ['e.exe', 2])]), 2));
test('non-array, null, or release without assets -> 0', () => {
  assert.equal(sumDownloads(null), 0);
  assert.equal(sumDownloads({ message: 'rate limited' }), 0);
  assert.equal(sumDownloads([{}, { assets: null }, null]), 0);
});
test('formatCount', () => {
  assert.equal(formatCount(0), '0');
  assert.equal(formatCount(999), '999');
  assert.equal(formatCount(1234), '1.2K');
  assert.equal(formatCount(12500), '12.5K');
  assert.equal(formatCount(1250000), '1.3M');
});
```

- [ ] **Step 2: Run to verify it fails**

Run (in `site/`): `node --test scripts/sum-downloads.test.mjs`
Expected: FAIL, module `sum-downloads.mjs` not found.

- [ ] **Step 3: Implement `sumDownloads(releases)` and `formatCount(n)` in `site/src/lib/sum-downloads.mjs`.** `sumDownloads` is defensive about shape (see tests): only finite numbers `>= 0` count, assets named `SHA256SUMS.txt` or ending `.sig` are skipped. `formatCount` returns the exact integer below 1,000, otherwise one decimal with `K` or `M` and a trailing `.0` dropped (`1000` → `1K`). `1250000` → `1.3M` pins round-half-up via `toFixed(1)`; if `toFixed` gives `1.2M` on your engine, round with `Math.round(n / 1e5) / 10`.

- [ ] **Step 4: Run to verify it passes**

Run: `node --test scripts/sum-downloads.test.mjs`
Expected: PASS, 5 tests.

- [ ] **Step 5: Commit**

```bash
git add site/src/lib/sum-downloads.mjs site/scripts/sum-downloads.test.mjs
git commit -m "feat(site): add download-count helpers"
```

---

### Task 2: Badge markup and live fetch

**Files:**
- Modify: `site/src/components/GetIt.astro` (insert after the `get-download` link, line ~19; style block)
- Modify: `site/src/pages/index.astro` (first `<script>` block, after the `fetch(... releases/latest ...)` chain, line ~139)

**Interfaces:**
- Consumes: `sumDownloads`, `formatCount` from `../lib/sum-downloads.mjs` (Task 1).

- [ ] **Step 1: Add the badge to `GetIt.astro`:** directly under the Download link, `<p class="meta get-count" data-downloads hidden><span data-downloads-n></span> downloads</p>`. Style `.get-count` with `margin-top: 12px` and the existing `.meta` look; no new colours.

- [ ] **Step 2: Fetch and reveal in `index.astro`'s first script.** Import `sumDownloads, formatCount`. Add a function that: reads `sessionStorage['fontcraft-downloads']` (number, in try/catch) else `fetch('https://api.github.com/repos/' + REPO + '/releases?per_page=100', { headers: { Accept: 'application/vnd.github+json' } })`, requires `r.ok`, passes the JSON to `sumDownloads`, and if the total is `> 0` sets `[data-downloads-n]` to `formatCount(total)`, removes `hidden` from `[data-downloads]`, and caches the number in `sessionStorage`. Any rejection is swallowed with `.catch(() => {})`, like the existing fetch. Add a comment: `shortcut: first 100 releases only, paginate if the repo ever has more`.

- [ ] **Step 3: Run the site gates**

Run (in `site/`): `node --test scripts/` then `node scripts/check-chrome.mjs` then `npm run build`
Expected: all tests pass; `chrome check passed`; `Complete!`

- [ ] **Step 4: Check it in the browser** with `npm run dev`:
  1. Page shows "N downloads" under the Download button (N matches the sum shown at `https://api.github.com/repos/mohseenjamall/FontCraft/releases`).
  2. In devtools, block `api.github.com` and reload: badge is absent, layout unchanged, Download button still works (it falls back to the releases page).
  3. At 375px width the badge does not overflow.

- [ ] **Step 5: Commit**

```bash
git add site/src/components/GetIt.astro site/src/pages/index.astro
git commit -m "feat(site): show total download count near Download"
```

---

## Self-Review

- **Spec coverage:** GitHub-sum count (Task 1+2), badge near Download (Task 2). The app ping is explicitly out of scope and gets its own plan.
- **Type consistency:** `sumDownloads(unknown): number`, `formatCount(number): string` are named identically in both tasks.
- **Review Focus:** 403/failure, zero total, bad `download_count`, checksum/sig files, non-array input, number formatting are covered by Task 1 tests or Task 2 step 4.
- **Not verified:** the exact `.dmg.sig` / `SHA256SUMS.txt` names are taken from the existing `pick-asset` tests, not from the live release.
