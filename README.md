# FontCraft

A fast, native font browser for Windows. See every font on your computer, each previewed in its
own typeface, with your own text and size, without a heavy web-based app.

Version 1.0.0

## Features

- **All your fonts in one list.** Shows system fonts, fonts installed for your user, fonts turned on by
  FontBase, and any folders you add, grouped by family.
- **Live preview.** Type your own sample text and pick a size from 6 to 180 px (starts at 42 px).
- **Right sample for each script.** A font that has no Latin letters shows a sample in its own
  writing system: Chinese, Japanese, Arabic, Hebrew, Devanagari, Thai and others.
- **Family details.** Click a family to see every style and a waterfall at 72, 48, 24 and 14 px.
- **Search and filter** by name or by where the fonts are stored.
- **Your folders are remembered.** Folders you add with **+ Add folder** come back the next time you
  open FontCraft.
- **Fast.** Fonts are scanned on several threads at once, and only the fonts on screen are loaded.
  About 500 font styles are listed and drawn in around 2 seconds. Compared with FontBase, it is
  ready about 4× sooner and uses about 7× less memory. See [EVALUATION.md](EVALUATION.md) for the
  full speed test (re-run it with `bench.ps1`).

## Build and run

You need [Rust](https://rustup.rs) installed.

```sh
cargo build --release
target/release/fontcraft.exe
```

Check the window without opening it in front of you: `fontcraft.exe --screenshot shot.png`
saves a picture of the window after it has loaded, then closes.

Run the tests with `cargo test`.

## How it's made

Written in Rust with [egui/eframe](https://github.com/emilk/egui) for the window and
[ttf-parser](https://github.com/harfbuzz/ttf-parser) to read font names. `MECHANICS.md` lists the
rules and numbers it follows and where each one came from.

## Not in version 1

Turning fonts on and off, collections and tags, the character map, OpenType feature controls and
variable-font sliders.
