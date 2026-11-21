# FontCraft

A fast, native font browser for Windows, macOS and Linux. See every font on your computer, each previewed in its
own typeface, with your own text and size, without a heavy web-based app.

Version 1.1.0

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

## Build and run

You need [Rust](https://rustup.rs) installed. On Linux, install the build libraries first:

```sh
sudo apt-get install -y libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libgl1-mesa-dev libgtk-3-dev
```

```sh
cargo build --release
```

Run it:

- **Windows:** `targeteleaseontcraft.exe`
- **macOS:** `target/release/fontcraft`. Release builds are unsigned: the first time, right-click
  the app and choose **Open** once.
- **Linux:** `target/release/fontcraft`

Check the window without opening it in front of you: `fontcraft --screenshot shot.png`
saves a picture of the window after it has loaded, then closes.

Run the tests with `cargo test`.

## Publishing an update

1. Raise `version` in `Cargo.toml` (for example `1.1.0`) and build with `cargo build --release`.
2. On GitHub, create a release whose tag is that version with a `v` in front (`v1.1.0`) and attach
   the program. Pushing a `v*` tag does this for you: `.github/workflows/release.yml` builds
   Windows, macOS (Apple silicon and Intel) and Linux binaries named
   `FontCraft-<version>-<platform>`, adds `SHA256SUMS.txt` and creates the release.
3. Every copy of FontCraft that's older shows the update bar the next time it starts.

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
