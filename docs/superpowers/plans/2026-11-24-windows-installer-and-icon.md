# Windows installer and icon Implementation Plan

> Execute task by task; steps use checkbox syntax.

**Goal:** Ship a per-user Windows installer and embed the app icon in the exe.
**Architecture:** `assets/icon.ico` plus `build.rs` (winresource) give the exe its icon; `main.rs` sets the AppUserModelID; `packaging/windows/FontCraft.iss` builds the installer in `release.yml`; the site and docs point at it.
**Spec:** docs/superpowers/specs/2026-11-24-windows-installer-and-icon-design.md

## Global Constraints
Branch `installer` from `main`. English UI and doc strings, sentence case, no em dashes. Binary <= 7.35 MiB, startup unchanged. The repo uses CRLF: keep each edited file's line endings. Nothing is pushed or released by these tasks.

## Review Focus
- The icon is really inside the exe (resource check) and the build fails loudly when it cannot be embedded.
- The installer needs no admin, upgrades over an older install, and uninstall leaves `%APPDATA%\FontCraft` and fonts alone.
- `pickAsset` still returns the portable exe when no setup asset exists (old releases), and the in-app updater's `asset_name` stays `windows-x64.exe` so it never downloads the setup file.

### Task 1: Icon and app identity
Files: `scripts/make-icon.py`, `assets/icon.ico`, `build.rs`, `Cargo.toml` (Windows-gated `[build-dependencies] winresource`, `windows-sys` feature `Win32_UI_Shell`), `src/main.rs`.
- [ ] `scripts/make-icon.py` (Pillow) writes `assets/icon.ico` with sizes 16, 24, 32, 48, 64, 128, 256 from `assets/app-art.png`; run it.
- [ ] `build.rs`: on a Windows target embed the icon and version info with `winresource`; fail with a clear message if compiling the resource fails.
- [ ] `main.rs`: first thing on Windows, `SetCurrentProcessExplicitAppUserModelID` with `"mohseenjamall.FontCraft"` (wide string; on error log to stderr and go on).
- [ ] Verify: `cargo build --release` with 0 warnings; the exe carries the icon (extract the associated icon and check it is not the default); size <= 7.35 MiB; `cargo test` passes.
- [ ] Commit.

### Task 2: Installer script and workflow
Files: `packaging/windows/FontCraft.iss`, `.github/workflows/release.yml`.
- [ ] Write the Inno script per the spec's Design bullets.
- [ ] `release.yml` Windows job: after naming the binary, install Inno Setup with choco if `iscc` is missing, run `iscc` with `/DAppVersion=$VERSION /DSourceExe=<exe path>`, copy the setup file into `dist/`.
- [ ] Verify locally if Inno Setup is available (else say so): silent install into a temp folder with `/VERYSILENT /DIR=...`, check the files, the Start shortcut and the uninstall entry, then uninstall; report what could not be checked.
- [ ] Commit.

### Task 3: Site and docs
Files: `site/src/lib/pick-asset.mjs` (and its test in `site/scripts/`), `site/src/components/GetIt.astro`, `README.md`, `MECHANICS.md`.
- [ ] Test first, then implement: Windows prefers `/windows-x64-setup\.exe$/` and falls back to `/windows-x64\.exe$/`.
- [ ] Update the texts: the installer is the main Windows download, the portable exe and zip stay, and "No installer yet" goes away.
- [ ] `node --test site/scripts/*.test.mjs`, `npm run build` in `site`, `node scripts/check-docs.mjs`.
- [ ] Commit.

### Task 4: Evidence
- [ ] `GATES-installer.md`: results of the verifications above, and what only a person can check (a real install on a clean user, the taskbar pin showing the icon, the SmartScreen text).
