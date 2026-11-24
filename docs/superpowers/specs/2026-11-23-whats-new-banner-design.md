# What's new banner

Date: 2026-11-23. Status: draft for the owner's approval. The owner chose "after updating" in chat.

## Intent and why

When someone updates FontCraft they should learn, once, what changed. Today the app only says "an update is available" and, after the update, shows nothing. A short "What's new" banner on the first launch of each new version tells users what we did and what was added, and makes updates feel worth it.

## Goals

- On the first launch of a newer version, show a banner "New in {version}" with 3 to 6 short bullets, a **Got it** button and a **Full release notes** link.
- Shows once per version: after it is shown or dismissed it never appears again for that version.
- Works offline (the notes ship inside the app) on Windows, macOS and Linux, after any update path (in-place replace, new .dmg, new binary).
- Writing the notes is part of releasing: a check fails the build if the current version has no notes.

## Non-goals

- No fetching of notes from GitHub, no Arabic or other translations yet, no pop-up window, no change to the "update available" banner.
- Not shown on a fresh install (nothing is "new" yet) or after a downgrade.

## Anti-goals

- Nagging: the same notes appearing every launch, or blocking work (it never covers the list and is dismissable).
- Silently showing nothing because someone forgot the notes (the build check prevents it).

## Approaches

1. **Notes bundled in the app, one file, shown once per version (chosen).** `assets/whats-new.md` holds a section per version; `include_str!` embeds it; the app parses the section for its own version. Offline, no network code, no new dependency.
2. Fetch the GitHub release body at launch: always current but needs network, can fail or be rate limited, and adds a request on start. Rejected.
3. Both (bundled, with the release body as an update): more code for little gain. Not now.

## Design

- `assets/whats-new.md`: `## 1.3.0` followed by `- ` bullets, newest first. Plain English, one line each, sentence case, no em dashes.
- `src/whats_new.rs`: `parse_notes(markdown: &str, version: &str) -> Option<Vec<String>>` (bullets of that version's section; `None` when absent or empty); `should_show(seen: Option<&str>, current: &str) -> bool` (true only when `seen` exists and `current` is a newer version, using the same version comparison as `update.rs`). Seen version stored in `seen_version.txt` next to `folders.txt` through `store::load_seen_version() -> Option<String>` / `store::save_seen_version(&str)`.
- Startup: if there is no seen file, write the current version and show nothing (fresh install). If the notes exist and `should_show`, show the banner and write the current version immediately, so it appears exactly once even if the app is closed without clicking.
- UI: a banner across the top of the content area, under the header, in both list and grid views, in the design-system style (paper card, hairline, cobalt "New in 1.3.0" title, bullets in `INK_DIM`, **Got it** primary button, **Full release notes** link that opens the release page for the version). The update banner stays above everything.
- `--whats-new` screenshot flag forces the banner with the current notes.
- `scripts/check-whats-new.mjs`: reads the version from `Cargo.toml` and fails if `assets/whats-new.md` has no `## {version}` section with at least one bullet; run it in `ci.yml` and in the release workflow before building.

## Calls (the owner can overturn)

- Marked as seen when shown, not when clicked.
- Only the current version's notes show, even if several versions were skipped; "Full release notes" covers the rest.
- English only for now; the string and notes file are ready for translation later.

## What is left to the builder

Exact layout and spacing of the banner, how "Full release notes" builds the release URL from the existing `update.rs` constants, and tests for `parse_notes`, `should_show` and the seen-version file.
