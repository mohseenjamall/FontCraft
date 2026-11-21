# Tell users when an update is out

**Goal:** when a newer FontCraft release is published on GitHub, every running copy shows a banner
that says an update is required, with a button that opens the download page.

## Rules
- **Where the version comes from:** `https://github.com/mohseenjamall/FontCraft/releases/latest`.
  GitHub answers with a redirect to `…/releases/tag/<tag>`; the tag is the latest version. No API,
  no JSON, no token. Only works while the releases are public: a private repo answers 404, and the
  check then quietly shows nothing.
- **How it's fetched:** once per start, on a background thread, with Windows' own `curl.exe`
  (`--max-time 10`, no window), so start-up never waits for the network and there's no new crate.
- **Newer:** compare `vMAJOR.MINOR.PATCH` (leading `v` optional, missing parts = 0, anything after
  `-` or `+` ignored) with the running version (`CARGO_PKG_VERSION`). Only strictly newer counts.
- **Trust:** the download button opens the redirect URL only if it starts with
  `https://github.com/mohseenjamall/FontCraft/releases/`.
- **Banner:** above the header, accent colour, can't be dismissed: "Update required: FontCraft
  <version> is available. Please update now." + **Download update**.
- **Offline / blocked / private / no curl:** no banner, no error, app works as before.
- `--screenshot` runs skip the network. `FONTCRAFT_LATEST=<tag>` stands in for the fetched tag (for
  checks and screenshots of the banner).

## Out of scope
Downloading and installing the update automatically; blocking the app until updated.
