# Plan: FontCraft website (Astro + GSAP)

Design: `docs/superpowers/specs/2026-11-21-fontcraft-site-design.md`

## Phase 0: Facts first (no code)
1. `gh release list -R mohseenjamall/FontCraft`: is a Release published, what is the asset name? Decides the Download button target.
2. Check `node -v` (Astro needs 18.20+/20+).
3. Re-read `EVALUATION.md` and copy the exact speed/memory numbers and test conditions for the Compare section.
4. List FontBase features for the comparison from FontBase's public pages. Anything unverified is left out, not guessed.

## Phase 1: Scaffold `site/`
1. Minimal Astro project (no UI framework), `gsap` pinned in `package.json`.
2. `site/.gitignore` (node_modules, dist). Nothing outside `site/` is touched.
3. Done when: `npm run build` produces a static `dist/`.

## Phase 2: Assets
1. Pick screenshots from `shots/` (main window, Arabic, turn on/off), crop, convert to WebP, with explicit width/height.
2. App icon / favicon from the app's icon.
3. Done when: total image weight is small enough that the page stays "light".

## Phase 3: Static page (no motion yet)
Components: Hero, Highlights, Compare, GetIt, Footer.
- Hero: "Faster. Lighter. Free.", Download for Windows + GitHub, "macOS and Linux soon".
- Compare: honest table, each row ✓ or "coming".
- GetIt: Windows live, macOS/Linux disabled with "soon".
- Mobile-first layout, readable with JS off.
- Done when: page reads correctly at phone and desktop width.

## Phase 4: Motion (GSAP + ScrollTrigger)
1. Section reveals, screenshot scroll movement, one live sample-text swap in Highlights.
2. `prefers-reduced-motion` disables all of it.
3. Load deferred, so first paint never waits for GSAP.

## Phase 5: Quality pass
1. `unslop` pass on copy and visuals; `frontend-design` review.
2. Check in the browser pane at 375px and desktop; check console is clean.
3. Lighthouse performance and accessibility (contrast, focus, alt text).

## Phase 6: Deploy (ask first)
GitHub Pages workflow for `site/`. Only added if the user wants it; publishing is the user's call.

## Risks
- MEDIUM: comparison claims about FontBase being wrong or outdated. Mitigation: Phase 0.4, cite only verifiable features.
- MEDIUM: Download link broken if no Release yet. Mitigation: link to Releases page.
- LOW: GSAP weight vs "lighter" message. Mitigation: defer, reduced-motion fallback.
- LOW: Node tooling in a Rust repo. Mitigation: everything isolated in `site/`.

Complexity: MEDIUM (about a day of focused work; most of it is copy and visual polish, not code).
