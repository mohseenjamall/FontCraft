epaint 0.37.0 with emilk/egui pull request #8577 (bidirectional text: shape RTL runs, place rows in
visual order, keep one glyph per char) applied to the `epaint` crate only. The egui half of that
pull request (selection painting) is not applied. Three hunks were merged by hand: the `unicode-bidi`
dependency in Cargo.toml, the `ShapedCluster` import in `text/fonts.rs`, and a `bidi_level` field in
the fontless-cluster `TextRun` in `text/text_layout.rs`. The pull request's new unit test was left out.
Remove this folder and the `[patch.crates-io]` entry in ../../Cargo.toml once egui ships the change.
