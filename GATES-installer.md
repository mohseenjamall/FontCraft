# Gates: Windows installer and icon

Branch `installer`. Checked by a temporary CI dry run (run 45485041829, windows-latest, Inno Setup from choco) plus local builds.

| Check | Result |
|---|---|
| `cargo test` (local) | 106 pass |
| Release build, 0 warnings | pass (Windows), CI green on Windows, macOS, Linux |
| Exe size | 7.324 MiB (limit 7.35) |
| Icon embedded | 5 sizes in `assets/icon.ico`; exe ProductName "FontCraft" |
| Installer builds (`iscc`, Inno 6, `x64compatible`) | pass |
| Silent install into a temp folder | exe, uninstaller and Start menu shortcut present |
| Uninstall entry | DisplayName "FontCraft version 0.0.1", Publisher mohseenjamall |
| Silent uninstall | exe and shortcut removed |
| Site picker | 22 tests pass; setup preferred, portable fallback |

Bugs the dry run found and fixed: `build.rs` referenced `winresource` on non-Windows hosts (broke macOS/Linux CI); the test version must be numeric for `VersionInfoVersion`.

Only a person can check: a real install on a clean user, the taskbar pin and Explorer showing the icon, the SmartScreen text (still "Unknown publisher", no signing), upgrading over an older install, uninstall keeping `%APPDATA%\FontCraft` and fonts.
