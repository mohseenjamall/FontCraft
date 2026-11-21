# FontCraft website: design

Size: small project (one static page). Short design, no separate builder check.

## Intent
A one-page site that says what FontCraft is: a faster, lighter, free replacement for FontBase.
Visitors should understand that in the first screen and be able to download it.

## Goals
- Hero: "Faster. Lighter. Free." with a one-line explanation, Download for Windows and GitHub buttons.
- Large real screenshot (from `shots/`), animated on scroll.
- Highlights: all fonts in one list, live preview, correct Arabic/other scripts, turn fonts on/off for every Windows app.
- Honest comparison table FontCraft vs FontBase. Speed and memory numbers come from `EVALUATION.md` (about 4x faster to ready, about 7x less memory). Missing features (Collections, Tags, Character map, variable-font sliders) are shown as "coming", not hidden.
- Get it: Windows available now, macOS and Linux marked "soon". Download links point to the GitHub Releases page of `mohseenjamall/FontCraft`.
- Footer line: open source, free, still in development.

## Non-goals
- No claim of being a "full" replacement today.
- No backend, accounts, analytics, blog, docs pages, pricing, email capture.

## Anti-goals (failure even if it works)
- Looks like generic AI output: gradient blobs, glass cards, three-icon feature grids, vague hype copy. Checked with the `unslop` skill.
- Any comparison claim that cannot be backed by `EVALUATION.md`, the README, or FontBase's own public pages.

## Approaches considered
1. Single HTML file, CSS + vanilla JS. Simplest, hosts anywhere.
2. Astro. Same output, more structure, needs a Node build in a Rust repo. Not needed yet.
3. Next.js. Far too much for a static page.

Chosen by the user: approach 2, **Astro**, plus **GSAP** (with ScrollTrigger) for motion.
Trade-offs accepted: a Node build step inside a Rust repo (kept isolated in `site/`, with its own `package.json`, never touching `app/` or `cargo`), and about 60 KB of GSAP, which slightly weakens the "lighter" message. Mitigations: Astro ships zero JS by default so only the motion script loads, deferred; the page must be fully readable without it.

## Decisions
- Location: `site/` in the repo as a self-contained Astro project (static output), deployable with GitHub Pages. Output stays static; no SSR, no UI framework (no React/Vue). (My call.)
- GSAP installed from npm at a pinned version and bundled by Astro. (My call.)
- Page split into small Astro components (Hero, Highlights, Compare, GetIt, Footer) so Docs or Changelog pages can be added later.
- Motion has a purpose: sections reveal on scroll, the screenshot moves with scroll, one preview line that swaps the sample text and font. No decorative loops. `prefers-reduced-motion` turns it all off. (My call.)
- Design skills: `frontend-design` for build, `unslop` as the anti-AI-look check. Dark, one accent color, mono labels in the style of the FilmCraft page but not a copy. (My call.)
- Language: English. Arabic version is a later option. (My call, easy to change.)
- Mobile layout works at phone width.

## Left to the builder
Exact accent color, fonts for the page, copy wording, screenshot selection and cropping, the FontBase column details (verify each against FontBase's public info before publishing).

## Open before publishing
- Confirm a published Release exists on GitHub and what the asset is called. Until then the Download button links to the Releases page.
