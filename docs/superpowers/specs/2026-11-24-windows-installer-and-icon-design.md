# Windows installer and app icon

Date: 2026-11-24. Status: approved by the owner in chat ("start with the installer and the icon, delegate the work").

## Intent and why
On Windows FontCraft is one loose `.exe`: it opens straight away, is never installed (no Start entry, no Apps list, no uninstall), the downloaded file has no icon, and the taskbar shows a generic icon. People expect an installed app with its picture everywhere.

## Goals
- A per-user installer `FontCraft-<version>-windows-x64-setup.exe`: no administrator prompt, installs to `%LOCALAPPDATA%\Programs\FontCraft`, adds a Start menu shortcut (optional desktop shortcut), appears in Settings > Apps with an uninstaller, offers "Launch FontCraft" at the end, and upgrades over an older install.
- The program's own icon is embedded in `fontcraft.exe` (a multi-size `.ico`), so the file, the shortcuts, the taskbar and Alt-Tab all show it.
- The taskbar groups and pins under one identity (an AppUserModelID shared by the program and the shortcut).
- The in-app "Update now" keeps working for an installed copy (the folder is writable).
- The release publishes the installer next to the portable `.exe`, `.zip`, dmg and Linux file; `SHA256SUMS.txt` covers it; the site's Windows button points at the installer.

## Non-goals
No code signing (the installer shows "Unknown publisher" like the exe), no machine-wide (Program Files) install, no MSIX, no auto-start, no change to macOS or Linux, no change to how the portable `.exe` self-updates. Uninstall keeps the person's settings in `%APPDATA%\FontCraft` and never touches their fonts.

## Anti-goals
An installer that needs administrator rights (it would break "Update now"); a second copy of the program fighting the portable one over the update; a build that silently ships without the icon.

## Approaches
1. **Inno Setup, per-user (chosen).** Mature, free, available on GitHub's Windows runners (or one `choco` away), supports upgrade in place, shortcuts with an AppUserModelID, and a silent mode (`/VERYSILENT`) that winget can use later.
2. WiX/MSI: heavier toolchain and per-machine by default. Rejected.
3. MSIX: needs signing to install at all. Rejected.
Icon: a `build.rs` using the `winresource` build-dependency (Windows only) to embed `assets/icon.ico` and version info.

## Design
- `assets/icon.ico` generated once from `assets/app-art.png` (16, 24, 32, 48, 64, 128, 256 px) and committed; `scripts/make-icon.py` regenerates it.
- `build.rs` (Windows only): embeds the icon, `ProductName=FontCraft`, `FileDescription`, and the version from `CARGO_PKG_VERSION`. A missing resource compiler is a clear build error, not a silent skip.
- `src/main.rs`: on Windows call `SetCurrentProcessExplicitAppUserModelID("mohseenjamall.FontCraft")` at start (needs the `Win32_UI_Shell` feature of `windows-sys`).
- `packaging/windows/FontCraft.iss`: fixed `AppId` GUID, `PrivilegesRequired=lowest`, `DefaultDirName={localappdata}\Programs\FontCraft`, `ArchitecturesInstallIn64BitMode=x64compatible`, version and source path passed with `/DAppVersion=... /DSourceExe=...`, shortcut with `AppUserModelID: "mohseenjamall.FontCraft"`, `CloseApplications=yes`, license page from `LICENSE`, output `FontCraft-<version>-windows-x64-setup.exe`.
- `release.yml`: the Windows job builds the installer after the exe (`iscc`, installing Inno Setup with choco if missing) and uploads it; the release job already checksums `FontCraft-*`.
- `site/src/lib/pick-asset.mjs`: Windows prefers `windows-x64-setup.exe` and falls back to the portable exe; its test follows. `GetIt.astro` text: installer first, portable still available.
- README and MECHANICS describe the installer, the portable file and the uninstall behaviour.

## What is left to the builder
Exact Inno script wording and the choco fallback; a rendering check of the icon sizes; verifying the installer by a silent install into a temporary folder and an uninstall.
