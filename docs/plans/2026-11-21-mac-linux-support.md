# macOS and Linux Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** FontCraft builds, runs, browses/previews fonts, turns them on/off, self-updates and is downloadable on macOS and Linux, as well as Windows.

**Architecture:** Browsing, preview and search are already portable egui code. What is Windows-only is (a) where folders live (`%WINDIR%`, `%APPDATA%`, `%LOCALAPPDATA%`), (b) font activation (registry + `AddFontResourceW`), (c) the updater's asset name / `curl.exe` / `NUL`, (d) the Arabic/Hebrew fallback fonts, (e) packaging. A new `platform.rs` owns every OS path; activation gets a Unix backend (symlinks into the user fonts folder + a fontconfig `rejectfont` file on Linux); the updater and release pipeline learn per-OS asset names. The Windows code paths stay as they are.

**Tech Stack:** Rust 2024, eframe/egui 0.37 (glow), GitHub Actions (ubuntu/macos/windows matrix), Astro site.

**Spec:** none separate; this plan is the design. Decisions it fixes: Linux can turn off user-installed fonts (fontconfig reject rule); macOS can only turn on/off fonts FontCraft linked itself (macOS has no per-user disable API), user and system fonts show as locked there; system fonts are locked everywhere; no font file is ever deleted or moved.

## Global Constraints

- No font file is ever deleted, moved or modified (existing rule, README).
- System fonts (`C:\Windows\Fonts`, `/usr/share/fonts`, `/usr/local/share/fonts`, `/System/Library/Fonts`, `/Library/Fonts`) are never touched; they show "System font" and stay locked.
- Windows behaviour and the existing Windows tests must not change or break.
- Cross-platform path logic is written as pure functions taking `home`/env values as arguments so it is unit-testable on the Windows dev machine; OS-specific effects are tested by CI on the real OS.
- Release assets: `FontCraft-<v>-windows-x64.exe`, `FontCraft-<v>-linux-x64`, `FontCraft-<v>-macos-arm64`, `FontCraft-<v>-macos-x64`, plus `SHA256SUMS.txt` listing all of them.
- Nothing is committed to git until the user approves (`git commit` only on request).

## Review Focus

- Folder containing a symlink cycle (`a/loop -> ..`): the scanner must finish, not recurse forever (new on Unix, `is_dir()` follows links).
- `$HOME` unset, or `XDG_DATA_HOME` / `XDG_CONFIG_HOME` set to a relative path: fall back to `~/.local/share` / `~/.config`; never write into the current directory.
- Font path containing `&`, `<`, spaces or non-ASCII (the reject file is XML) and containing `* ? [` (fontconfig globs): XML-escape the first group, refuse the second with a clear error.
- A linked font whose source file was deleted (dangling symlink): turn-on/off and listing must not panic; the dangling link is removed on turn off.
- `fc-cache` not installed, or the user fonts folder missing: activation still succeeds (best-effort refresh) and creates the folder.

---

### Task 1: CI that builds and tests on all three OSes

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Produces: a CI job named `test` with matrix `ubuntu-latest, macos-latest, windows-latest`, which every later task is verified against.

- [ ] **Step 1: Write `ci.yml`**: on `push` and `pull_request`; steps: checkout, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, on ubuntu `sudo apt-get update && sudo apt-get install -y libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libgl1-mesa-dev libgtk-3-dev`, then `cargo test --locked` and `cargo build --release --locked`.
- [ ] **Step 2: Try a local cross-check first** (cheap feedback, linking is not needed): `rustup target add x86_64-unknown-linux-gnu x86_64-apple-darwin` then `cargo check --target x86_64-unknown-linux-gnu` and `cargo check --target x86_64-apple-darwin`. Expected: shows the Unix compile errors/warnings later tasks must fix (build scripts needing pkg-config may fail locally; then rely on CI).
- [ ] **Step 3: Push a branch and confirm** the `windows-latest` job is green and the other two show their current failures. Expected: Windows PASS.
- [ ] **Step 4: Commit** `ci: build and test on linux, macos and windows`.

---

### Task 2: One module for every OS path

**Files:**
- Create: `src/platform.rs` (declare `mod platform;` in `src/main.rs`)
- Modify: `src/store.rs:4-6` (data dir), `src/scan.rs:107-118` (`system_folders`), `src/fonts.rs:79-86` (`add_system_fallback`)
- Test: `#[cfg(test)] mod tests` in `src/platform.rs`

**Interfaces:**
- Produces (all in `platform`, all `pub`):
  - `fn app_data_dir() -> Option<PathBuf>`: Windows `%APPDATA%\FontCraft`; macOS `~/Library/Application Support/FontCraft`; Linux `$XDG_CONFIG_HOME` (absolute only) else `~/.config`, then `/FontCraft`.
  - `fn user_fonts_dir() -> Option<PathBuf>`: Windows `%LOCALAPPDATA%\Microsoft\Windows\Fonts`; macOS `~/Library/Fonts`; Linux `$XDG_DATA_HOME` (absolute only) else `~/.local/share`, then `/fonts`.
  - `fn system_font_dirs() -> Vec<PathBuf>`: per the Global Constraints list for the current OS.
  - `fn fallback_fonts() -> Vec<PathBuf>`: Arabic/Hebrew-capable files that exist. Windows `segoeui.ttf`, `tahoma.ttf`; macOS `/System/Library/Fonts/Supplemental/Arial.ttf`, `/System/Library/Fonts/Supplemental/Tahoma.ttf` (filter on `exists()`); Linux first existing of `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`, `/usr/share/fonts/TTF/DejaVuSans.ttf`, `/usr/share/fonts/noto/NotoSansArabic-Regular.ttf`.
  - Pure helpers (compiled on every OS, used by the above): `fn linux_dir(home: Option<&Path>, xdg: Option<&Path>, home_rel: &str) -> Option<PathBuf>` returns `xdg` if absolute, else `home.join(home_rel)`, else `None`.
- Consumes: nothing new.

- [ ] **Step 1: Write failing tests** in `platform.rs`: `linux_dir_prefers_absolute_xdg`, `linux_dir_ignores_relative_xdg` (`Some("rel")` → `home/.local/share`), `linux_dir_none_without_home_or_xdg`, `linux_dir_appends_nothing_when_xdg_used` (callers append `fonts`/`FontCraft`).
- [ ] **Step 2: Run** `cargo test platform` → FAIL (module missing).
- [ ] **Step 3: Implement `platform.rs`** with the signatures above; the OS-specific functions use `cfg!(windows)`/`cfg!(target_os = "macos")` branches over `std::env::var_os("HOME")` etc. and call `linux_dir`.
- [ ] **Step 4: Run** `cargo test platform` → PASS.
- [ ] **Step 5: Rewire callers.** `store::dir()` → `platform::app_data_dir()`. `scan::system_folders()` → labelled list: each `system_font_dirs()` entry as `"System"` plus `user_fonts_dir()` as `"User installed"` (keep Windows output identical: same two entries, same order). `fonts::add_system_fallback` → loop `platform::fallback_fonts()` (font name = file name). Leave `disabled::data_path` (Windows registry data) alone.
- [ ] **Step 6: Make the Windows-only test OS-neutral**: `scan.rs` test `reads_names_from_a_system_font` gets `#[cfg(windows)]`.
- [ ] **Step 7: Run** `cargo test` → PASS on Windows; push and check CI `cargo check` errors from Task 1 shrink.
- [ ] **Step 8: Commit** `refactor: move OS paths into platform module`.

---

### Task 3: Scanner survives symlink loops

**Files:**
- Modify: `src/scan.rs:49-67` (`collect_files`)
- Test: `src/scan.rs` tests (`#[cfg(unix)]`)

**Interfaces:**
- Consumes: nothing.
- Produces: `collect_files(dir: &Path, source: usize, out: &mut Vec<(usize, PathBuf)>)` unchanged signature; internally tracks visited canonical directories.

- [ ] **Step 1: Write failing test** `#[cfg(unix)] fn scan_finishes_on_symlink_loop`: temp dir with `fonts/a.ttf` (copy any bytes; parsing may fail, only collection is tested) and `fonts/loop -> ..` via `std::os::unix::fs::symlink`; assert `collect_files` returns exactly one `a.ttf` entry and terminates.
- [ ] **Step 2: Run** it on Linux CI (or WSL) → hangs/overflows or duplicates.
- [ ] **Step 3: Implement**: pass a `&mut HashSet<PathBuf>` of `fs::canonicalize` results, skip a directory already visited; keep behaviour for plain trees.
- [ ] **Step 4: Run** `cargo test scan` → PASS on all three CI jobs.
- [ ] **Step 5: Commit** `fix: stop scanner following symlink loops`.

---

### Task 4: Updater works on every OS

**Files:**
- Modify: `src/update.rs:51-66` (`curl`), `:68-72` (`fetch`), `:94-97` (`asset_name`), `:125-139` (`install`), `:160-166` (`open`), tests `:166-273`

**Interfaces:**
- Produces: `fn asset_name(version: &str) -> String` returns `FontCraft-{v}-windows-x64.exe` on Windows, `FontCraft-{v}-linux-x64` on Linux, `FontCraft-{v}-macos-arm64` / `-macos-x64` on macOS by `cfg!(target_arch)`.
- Consumes: nothing.

- [ ] **Step 1: Write failing tests**: `asset_name_matches_this_os` asserting the exact string for the current `cfg` (three `#[cfg]` variants); `fetch_args_use_null_device` is not unit-testable, so cover it in Step 6.
- [ ] **Step 2: Run** `cargo test asset_name` → FAIL on Linux/macOS CI.
- [ ] **Step 3: Implement** `asset_name` as above. In `fetch`, replace the hard-coded `"NUL"` with `if cfg!(windows) { "NUL" } else { "/dev/null" }`. Update the `shortcut:` comment (curl ships with Windows 10 1803+, macOS and nearly all Linux).
- [ ] **Step 4: Executable bit.** In `install`, after the checksum passes and before `self_replace`, on Unix `std::fs::set_permissions(&exe, Permissions::from_mode(0o755))` (a downloaded file is not executable; `self_replace` copies from it). Test `#[cfg(unix)] install_marks_download_executable` is replaced by a pure helper `fn make_executable(path: &Path) -> io::Result<()>` with its own test asserting `mode & 0o111 != 0`.
- [ ] **Step 5: `open`**: opener `explorer` on Windows, `open` on macOS, `xdg-open` otherwise. Keep the `trusted()` check first.
- [ ] **Step 6: Run** `cargo test update` locally and on CI → PASS on all three.
- [ ] **Step 7: Commit** `feat: self-update on macos and linux`.

---

### Task 5: Turning fonts on and off on macOS and Linux

**Files:**
- Create: `src/activate_unix.rs` (`#[cfg(unix)] mod activate_unix;` in `main.rs`)
- Modify: `src/activate.rs` (`norm`, `is_system_font`, `FONTS_KEY` → `key()`, the `#[cfg(not(windows))]` stubs at `:249-270`), `src/store.rs:15-18` (`disabled_list`)
- Test: tests inside `src/activate_unix.rs` using a temp dir for both the links folder and the conf file

**Interfaces:**
- Produces in `activate`:
  - `pub fn key() -> String`: Windows `FONTS_KEY`; Unix `platform::user_fonts_dir()/FontCraft` (the *links folder*, as a string). Replaces the `FONTS_KEY` constant for callers (`app.rs:129,161`).
  - `pub fn norm(path: &Path) -> PathBuf`: lowercases and `/`→`\` only on Windows; identity elsewhere.
  - `pub fn lock_reason(path: &Path) -> Option<&'static str>`: `Some` when the file cannot be toggled. Windows: file directly in `%WINDIR%\Fonts` → "Installed for every user in Windows\\Fonts; FontCraft never changes these". Unix: under `system_font_dirs()` → "Installed for every user; FontCraft never changes these"; on macOS also under `user_fonts_dir()` but outside the links folder → "macOS manages fonts in ~/Library/Fonts; use Font Book to turn these off". Keep `is_system_font` for Windows.
  - `pub fn link_name(path: &Path) -> String`: `<stem>-<fnv1a-32 of path bytes as 8 hex>.<ext>`.
- Produces in `activate_unix` (same shapes as the Windows `imp`, so `activate` re-exports them):
  - `pub fn read_active(links: &str) -> HashSet<PathBuf>`: targets of the symlinks in `links` (dangling ones skipped).
  - `pub fn read_disabled(conf: &Path) -> HashSet<PathBuf>`: paths in the FontCraft reject file; empty if missing/unreadable; always empty on macOS.
  - `pub fn turn_on_in(links: &str, conf: &Path, paths: &[PathBuf]) -> Result<usize, String>`: for a path inside `user_fonts_dir()` (Linux): remove its rule from `conf`; for any other path: create `links/link_name(path)` → `path` (create `links` first; existing correct link counts as done). Returns the number changed, then `refresh()`.
  - `pub fn turn_off_in(...)` same args: owned link → remove the symlink (never the target); path inside `user_fonts_dir()` on Linux → add a rule to `conf`; a dangling link is removed too; anything else → `Err("… can't be turned off here")`.
  - `fn refresh(links: &Path)`: Linux only, runs `fc-cache -f <links parent>` ignoring errors and a missing binary; macOS no-op.
  - `fn write_reject_conf(conf: &Path, paths: &BTreeSet<PathBuf>) -> Result<(), String>`: writes the file below atomically (temp file + rename), creating parent folders; `fn parse_reject_conf(text: &str) -> BTreeSet<PathBuf>` is its exact inverse.
- Conf format (exact):
  ```xml
  <?xml version="1.0"?>
  <!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
  <fontconfig>
  <!-- Written by FontCraft: fonts you turned off. -->
  <selectfont><rejectfont>
  <glob>/home/me/.local/share/fonts/A &amp; B.ttf</glob>
  </rejectfont></selectfont>
  </fontconfig>
  ```
- `store::disabled_list()` returns, on Linux, `~/.config/fontconfig/conf.d/99-fontcraft.conf` (via `XDG_CONFIG_HOME`); on macOS `app_data_dir()/disabled.tsv` (unused but harmless); Windows unchanged.

- [ ] **Step 1: Write failing tests** in `activate_unix.rs` (temp dir helper like `activate::tests::Scratch`):
  - `turn_on_creates_link_and_read_active_sees_target`
  - `turn_on_twice_is_idempotent` (returns Ok, one link)
  - `turn_off_removes_link_not_target` (target file still exists)
  - `turn_off_dangling_link_succeeds` (delete the target first)
  - `reject_conf_round_trip_with_special_chars`: paths `/f/A & B <1>.ttf` and `/f/خط.ttf` survive `write_reject_conf` → `parse_reject_conf`, and the file contains `&amp;`
  - `turn_off_refuses_glob_characters`: path with `[` → `Err` mentioning the character, conf untouched
  - `turn_off_user_font_adds_rule_and_turn_on_removes_it` (Linux-only, passes `user_fonts` root explicitly through an inner fn `turn_off_with(user_fonts: &Path, …)` so the test needs no `$HOME`)
  - `missing_links_folder_is_created`
  - `lock_reason_system_font_is_some` in `activate.rs` with `/usr/share/fonts/x.ttf` (`#[cfg(unix)]`)
  - `link_name_differs_for_same_file_in_two_folders` (runs on every OS)
- [ ] **Step 2: Run** `cargo test activate` → FAIL (CI Linux/macOS, or WSL).
- [ ] **Step 3: Implement** `activate_unix.rs` per the Interfaces; macOS has the same code but the reject-rule branch returns `Err(lock reason)` instead.
- [ ] **Step 4: Wire `activate.rs`**: delete the `#[cfg(not(windows))]` "works on Windows only" stubs; on Unix `pub use activate_unix::{read_active, read_disabled, turn_off_in, turn_on_in}`; add `key()`, `norm`, `lock_reason`, `link_name` as above; keep the Windows `imp` untouched.
- [ ] **Step 5: Run** `cargo test` locally (Windows still PASS) and on CI → all three PASS.
- [ ] **Step 6: Commit** `feat: turn fonts on and off on macos and linux`.

---

### Task 6: App uses the new activation API and lists linked fonts

**Files:**
- Modify: `src/app.rs:49-60` (`all_active`, `fontbase_files`), `:96-135` (`new`), `:154-181` (`can_toggle`, `toggle`), `:515-535` (button text and hover), `src/scan.rs:120-142` (`fontbase_activated`)
- Test: `src/app.rs` tests (new `#[cfg(unix)]` cases using `FontApp::for_test`)

**Interfaces:**
- Consumes: `activate::{key, norm, lock_reason, read_active, read_disabled, turn_on_in, turn_off_in}` from Task 5; `platform::user_fonts_dir`.
- Produces: `scan::fontbase_activated() -> Vec<PathBuf>`: on Unix, the existing targets in the links folder (so a linked font from an unscanned folder stays listed as "Activated by FontCraft"); `app::all_active` returns true for a face whose `norm(path)` is in `active`, **or on Unix** whose path lies in `user_fonts_dir()` (outside the links folder) and is not in `disabled` — fonts there are on by default.

- [ ] **Step 1: Write failing tests**: `unix_user_font_is_on_until_rejected`; `unix_system_font_cannot_toggle` (`can_toggle` false, path `/usr/share/fonts/a.ttf`); `unix_toggle_links_font_from_added_folder` (temp links dir via `toggle_in`, then `active` contains the path).
- [ ] **Step 2: Run** `cargo test app` on CI → FAIL.
- [ ] **Step 3: Implement**: replace `activate::FONTS_KEY` with `activate::key()` (3 call sites); `can_toggle` → `lock_reason(..).is_none()` for all faces; the button hover uses the first non-`None` `lock_reason` (text: "System font: needs administrator" on Windows, "Locked" elsewhere); `all_active` extended as above; `scan::fontbase_activated` Unix body reads the links folder, label "Activated by FontBase" renamed "Turned on by FontCraft" on Unix. The scanner must skip the links folder so a linked font isn't listed twice: in `collect_files` ignore a directory whose path equals `activate::key()`.
- [ ] **Step 4: Run** `cargo test` → PASS locally and on all CI jobs.
- [ ] **Step 5: Manual smoke on Linux** (VM or CI artifact): run, add a folder, Turn on a font, `fc-list | grep <name>` shows it; Turn off, it is gone; file still on disk. Record the result in the PR; **not verifiable on macOS without a Mac**: first release is "beta" for macOS (symlinks in `~/Library/Fonts` are expected to work, unconfirmed).
- [ ] **Step 6: Commit** `feat: show and toggle fonts on macos and linux`.

---

### Task 7: Release pipeline for all platforms

**Files:**
- Create: `.github/workflows/release.yml`
- Modify: `README.md:3,18-22,42-45`, `scripts/check-update-docs.mjs` (if it pins the Windows asset name)

**Interfaces:**
- Consumes: `asset_name` formats from Task 4.
- Produces: on tag `v*`, a GitHub release with the four assets and `SHA256SUMS.txt` (lines `<hex>  <file>`, exactly what `update::expected_hash` parses).

- [ ] **Step 1: Write `release.yml`**: matrix `{os: windows-latest, target: x86_64-pc-windows-msvc, asset: windows-x64.exe}`, `{ubuntu-22.04, x86_64-unknown-linux-gnu, linux-x64}` (22.04 for an older glibc), `{macos-14, aarch64-apple-darwin, macos-arm64}`, `{macos-13, x86_64-apple-darwin, macos-x64}`; each builds `cargo build --release --locked --target <t>`, copies the binary to `FontCraft-${version}-${asset}` (version = tag without `v`), uploads as artifact. A final `release` job downloads all, runs `sha256sum FontCraft-* > SHA256SUMS.txt`, and `gh release create "$TAG" FontCraft-* SHA256SUMS.txt`. Keep the existing `.zip` for Windows if the site still links it.
- [ ] **Step 2: Verify the name/format contract** with a throwaway tag on a fork or draft: download `SHA256SUMS.txt`, run `cargo test expected_hash` and the in-app update path against it (`FONTCRAFT_LATEST=<tag>`).
- [ ] **Step 3: README**: say "Windows, macOS and Linux"; list Linux build deps from Task 1; describe Linux/macOS activation limits (macOS locks fonts it did not link; Linux apps may need a restart to see changes); replace `fontcraft.exe` run instructions with per-OS ones; macOS first run: unsigned build, right-click → Open once.
- [ ] **Step 4: Run** `node scripts/check-update-docs.mjs` and the other `scripts/*.mjs` checks you normally run → PASS.
- [ ] **Step 5: Commit** `ci: release builds for windows, linux and macos`.

---

### Task 8: Website offers the right download

**Files:**
- Modify: `site/src/pages/index.astro:~34` (asset matcher), `site/src/components/GetIt.astro:24-28`, Hero/Download button label, `site/GATES.md` if it lists the asset

**Interfaces:**
- Consumes: asset names from Task 7. Produces: Download buttons resolve to the asset for the visitor's OS (Windows → `windows-x64.exe`; macOS → `macos-arm64` unless `navigator.userAgent` clearly Intel, then `macos-x64`; Linux → `linux-x64`); unknown OS falls back to `releases/latest`. All four files listed under "Get it".

- [ ] **Step 1: Write a node check** `site/scripts/pick-asset.test.mjs` (or reuse the site's test style) for a pure `pickAsset(assets: {name:string}[], platform: 'windows'|'mac'|'linux'|'other', arm: boolean) -> asset | undefined` with: windows→exe, linux→`linux-x64`, mac+arm→`macos-arm64`, mac+intel→`macos-x64`, other→`undefined`, missing asset→`undefined`.
- [ ] **Step 2: Run** `node --test site/scripts/pick-asset.test.mjs` → FAIL.
- [ ] **Step 3: Implement `pickAsset`** in a small module imported by `index.astro`'s release script; replace the `/windows-x64\.exe$/` find; update copy ("Download for macOS" etc.) and the version-substitution to keep working.
- [ ] **Step 4: Run** the node test → PASS; `npm run build` in `site/` → PASS; in the browser pane with the Windows UA, button still resolves to the `.exe`.
- [ ] **Step 5: Commit** `feat(site): per-OS download button`.

---

## Self-review

- **Coverage:** paths (2), scanning (3), updater (4), activation (5, 6), fallback fonts (2), CI (1), packaging (7), site/docs (7, 8). Window/AlwaysOnBottom in `main.rs` is already `cfg(windows)`; no change needed.
- **Names:** `key()`, `norm`, `lock_reason`, `link_name`, `read_active/read_disabled/turn_on_in/turn_off_in` are identical in Tasks 5 and 6.
- **Risks the user must know:** macOS behaviour (symlinked fonts in `~/Library/Fonts`, Gatekeeper on an unsigned binary) is untested until someone runs it on a Mac; Linux running apps usually need a restart to see font changes; reject rules affect fontconfig apps only (not every Linux app).

---

### Task 9 (added later by user request): Support link and app picture inside the app

**Files:**
- Create: `assets/app-art.png` (256x256, made from `site/src/assets/engraving-hero.jpg`, the user's engraving of the "F"), `src/support.rs`
- Modify: `src/main.rs` (window icon via `ViewportBuilder::with_icon`, `mod support;`), `src/app.rs` (support card in the side panel/bottom), `src/update.rs` only if reusing `open`
- Test: tests in `src/support.rs`

**Interfaces:**
- Produces: `pub const SUPPORT_URL: &str = "https://buymeacoffee.com/mohseenjamalll"`; `pub fn open_support() -> Result<(), String>` (opens only `SUPPORT_URL`, same openers as `update::open`: explorer / open / xdg-open); `pub fn app_icon() -> egui::IconData` decoded from `include_bytes!("../assets/app-art.png")` with the `image` crate (png feature already enabled).
- [ ] **Step 1: Failing tests**: `support_url_is_the_buymeacoffee_page` (exact string); `app_icon_is_256_square_rgba` (width == height == 256, `rgba.len() == 256*256*4`); `open_support_refuses_other_urls` is not needed (no input).
- [ ] **Step 2: Run** `cargo test support` → FAIL. **Step 3: Implement**; the card shows the picture (64px, rounded) plus "Support FontCraft" and a "Buy me a coffee" button calling `open_support`, errors shown through the existing `notice`. **Step 4: Run** `cargo test` → PASS; screenshot check with `--screenshot`. **Step 5: Commit** `feat: support link and app picture`.
- Not covered: embedding the icon into the `.exe` file itself (needs a Windows resource build step); the window/taskbar icon only.
