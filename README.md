# FontCraft

A fast, native font browser for Windows, macOS and Linux. See every font on your computer, each previewed in its
own typeface, with your own text and size, without a heavy web-based app.

Version 1.2.0

## Features

- **All your fonts in one list.** Shows system fonts, fonts installed for your user, fonts turned on by
  FontBase, and any folders you add, grouped by family.
- **Live preview.** Type your own sample text and pick a size from 6 to 180 px (starts at 42 px).
- **Right sample for each script.** A font that has no Latin letters shows a sample in its own
  writing system: Chinese, Japanese, Arabic, Hebrew, Devanagari, Thai and others.
- **Family details.** Click a family to see every style and a waterfall at 72, 48, 24 and 14 px.
- **Search and filter** by name or by where the fonts are stored.
- **Turn fonts on and off.** Pick a family and click **Turn on** or **Turn off**. It works for fonts
  from a folder you added, fonts installed for your user, and fonts FontBase turned on. Every Windows
  app (Word, Photoshop, Notepad…) sees the change right away, and it lasts after you sign out or
  restart. Turning a font off never deletes its file: FontCraft keeps its exact Windows settings and
  puts them back when you turn it on again. Fonts that are on show an **ON** badge in the list.
  System fonts (in `C:\Windows\Fonts`, installed for every user) stay locked and say "System font:
  needs administrator". If one file holds several families, they change together. After you turn
  off a FontBase font, FontBase may still show it as on until you open it there. On macOS, fonts
  FontCraft did not link itself stay locked. On Linux, a running app may need a restart before it
  sees a change.
- **Your folders are remembered.** Folders you add with **+ Add folder** come back the next time you
  open FontCraft.
- **Update notice.** Each time FontCraft starts it checks, in the background, whether a newer
  release is on GitHub. If there is one, a purple bar across the top says "Update required" with a
  **Download update** button that opens the release page. Offline or blocked: nothing is shown and
  the app works as usual.
- **Fast.** Fonts are scanned on several threads at once, and only the fonts on screen are loaded.
  About 500 font styles are listed and drawn in around 2 seconds. Compared with FontBase, it is
  ready about 4× sooner and uses about 7× less memory. See [EVALUATION.md](EVALUATION.md) for the
  full speed test (re-run it with `bench.ps1`).

## Install on macOS

macOS 11 or later, Apple Silicon or Intel.

1. Download `FontCraft-<version>-macos.dmg` from the
   [latest release](https://github.com/mohseenjamall/FontCraft/releases/latest).
2. Open it and drag **FontCraft** onto **Applications**.
3. Open FontCraft from Applications or Launchpad.

When a newer version is out, the update bar's **Download update** button downloads the new `.dmg`;
install it the same way (replace the old FontCraft when asked).

## Build and run

You need [Rust](https://rustup.rs) installed. On Linux, install the build libraries first:

```sh
sudo apt-get install -y libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libgl1-mesa-dev libgtk-3-dev
```

```sh
cargo build --release
```

Run it:

- **Windows:** `target\release\fontcraft.exe`
- **macOS:** `target/release/fontcraft`
- **Linux:** `target/release/fontcraft`

A downloaded Linux release binary needs `chmod +x FontCraft-*` first.

Check the window without opening it in front of you: `fontcraft --screenshot shot.png`
saves a picture of the window after it has loaded, then closes.

Run the tests with `cargo test`.

## Publishing an update

1. Raise `version` in `Cargo.toml` (for example `1.1.0`) and build with `cargo build --release`.
2. On GitHub, create a release whose tag is that version with a `v` in front (`v1.1.0`) and attach
   the program. Pushing a `v*` tag does this for you: `.github/workflows/release.yml` builds
   `FontCraft-<version>-windows-x64.exe` (and `.zip`), `FontCraft-<version>-linux-x64` and
   `FontCraft-<version>-macos.dmg` (one universal app for Apple Silicon and Intel, built by
   `.github/workflows/macos-build.yml`), adds `SHA256SUMS.txt` and creates the release.
3. Every copy of FontCraft that's older shows the update bar the next time it starts. Windows and
   Linux replace themselves; macOS opens the new `.dmg` (swapping the program inside a signed app
   would stop it from opening).

### Maintainers: macOS signing

Without these repository secrets the macOS job still builds the `.dmg`, but unsigned, and macOS
blocks it on other Macs. With them it is signed with your Developer ID and notarized by Apple, so
it opens with no warning. Set them once (Settings > Secrets and variables > Actions, or `gh`):

| Secret | What it is |
|---|---|
| `MACOS_CERTIFICATE` | your Developer ID Application certificate as a base64 `.p12` |
| `MACOS_CERTIFICATE_PASSWORD` | the password you gave the `.p12` when exporting it |
| `MACOS_SIGN_IDENTITY` | the full name, e.g. `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_ID` | the Apple ID email of your developer account |
| `APPLE_TEAM_ID` | your 10-character Team ID (developer.apple.com > Membership) |
| `APPLE_APP_PASSWORD` | an app-specific password for notarization |

1. At [developer.apple.com](https://developer.apple.com/account/resources/certificates/list)
   create a **Developer ID Application** certificate and install it in Keychain Access.
2. In Keychain Access, right-click the certificate (with its private key), **Export** it as
   `cert.p12` and choose a password. Then run, on your Mac:

   ```sh
   base64 -i cert.p12 | gh secret set MACOS_CERTIFICATE
   gh secret set MACOS_CERTIFICATE_PASSWORD
   gh secret set MACOS_SIGN_IDENTITY   # paste: Developer ID Application: Your Name (TEAMID)
   ```

   `security find-identity -v -p codesigning` prints the exact identity name.
3. At [appleid.apple.com](https://appleid.apple.com) > Sign-In and Security > App-Specific
   Passwords, create one for "FontCraft notarization", then:

   ```sh
   gh secret set APPLE_ID
   gh secret set APPLE_TEAM_ID
   gh secret set APPLE_APP_PASSWORD
   ```

   Each `gh secret set` asks for the value, so it never lands in your shell history. Delete
   `cert.p12` afterwards.

The check reads `https://github.com/mohseenjamall/FontCraft/releases/latest`, which only works
while the releases are **public**: with a private repository GitHub answers "not found" and no
copy will ever see the update bar. To test the bar without publishing, start FontCraft with the
environment variable `FONTCRAFT_LATEST=v9.9.9`.

## How it's made

Written in Rust with [egui/eframe](https://github.com/emilk/egui) for the window and
[ttf-parser](https://github.com/harfbuzz/ttf-parser) to read font names. `MECHANICS.md` lists the
rules and numbers it follows and where each one came from.

## Not in version 1

Collections and tags, the character map, OpenType feature controls and variable-font sliders.

## License

FontCraft is released under the [MIT license](LICENSE). The bundled UI fonts (Archivo, IBM Plex Mono) are under the SIL Open Font License.
