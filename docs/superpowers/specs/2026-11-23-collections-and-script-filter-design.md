# Collections and script-aware filtering

Date: 2026-11-23. Status: draft for the owner's approval. Requested by users; the owner approved the section proposal and the filter behaviour in chat and confirmed the filter is about the **preview text box**.

## Intent and why

Users with big font libraries cannot find what they need by script. Two things they asked for:

1. Sidebar sections that group fonts: **Arabic**, **English**, **Variable fonts**, and fonts with **more than 3 weights**.
2. When someone types in the preview box, only fonts that can really draw that text should be listed: Arabic text lists Arabic-capable fonts only, English text lists English-capable fonts only. Today every font is listed and the ones without glyphs are dimmed with "no glyphs for this text".

Both need to know which writing systems each font supports, so they are one project.

## Goals

- Sidebar group **Collections** under the font sources: Arabic, English, Variable, Many weights, each with a count; picking one filters the list and the grid.
- Preview-text filter in the list view: fonts must support every writing system present in the typed text; clearing the text brings every font back.
- A small notice above the list while the filter is active ("Showing fonts that support Arabic") so fonts never vanish unexplained.
- No visible slowdown: scan time, startup, memory and typing latency stay within the existing budget (ready <= 2.2 s, working set <= 110 MB, binary <= 7.35 MiB; the grid view's 112 MB is a known open item).

## Non-goals

- No sections beyond the four; no "bilingual" section (an Arabic font that also has Latin simply appears in both Arabic and English).
- No change to the name search box, no saved index of the library, no changes to font activation (that request waits for the owner's contact).
- The filter does not apply in the grid (its preview box is dimmed there).

## Anti-goals

- A font that supports a script being hidden by a wrong or missing test letter.
- Fonts disappearing with no explanation.
- Work on every keystroke that loads fonts or scans files.

## Constraints

Existing code: `scan.rs` builds one `Face` per font face and already tests glyphs once per script with `samples::script_of` (first match only); `Family` groups faces by name in `app.rs`; `visible()` filters by the name search and the chosen source; the sidebar draws source rows with `source_row`. Colours, spacing and widgets come from `theme` and `widgets`; sentence case; no new dependency (`ttf-parser` already exposes `is_variable()`).

## Approaches

1. **Precompute at scan time (chosen).** Each `Face` gets `scripts: u16`, a bitmask over the 16 entries of `samples::SCRIPTS` (bit set when the face has that script's test letter), and `variable: bool` (`ttf_parser::Face::is_variable()`). A family ORs its faces. Collections and the preview filter become integer mask tests, so typing costs nothing. Scan cost: 16 glyph lookups per face instead of stopping at the first hit.
2. Check on demand with `has_glyphs` while typing: needs every font loaded into egui, which defeats the app's on-demand loading and its memory and startup advantage. Rejected.
3. Persist a script index on disk to skip scanning: more moving parts for a scan that already takes about a second. Not now.

## Design

- `samples.rs`: `pub fn script_mask(has: impl Fn(char) -> bool) -> u16`; `pub fn scripts_in(text: &str) -> u16` maps each letter of typed text to a script index by Unicode block (Latin letters, Arabic blocks 0600-06FF, 0750-077F, FB50-FDFF, FE70-FEFF, Hebrew, CJK ideographs, Hiragana/Katakana, Devanagari, Cyrillic, Greek, Khmer, Lao, Thai, Tamil, Myanmar, Bengali, Armenian, Georgian). Digits, spaces, punctuation and symbols map to nothing. `script_of` stays for choosing the default sample.
- Constants: Latin = bit 0 (the "English" collection), Arabic = bit 4.
- `Face` gains `scripts` and `variable`; `Family` gains `scripts` (OR of faces), `variable` (any face), and `weight_count` (distinct weight values among its faces). "More than 3 weights" means `weight_count > 3`, so a Regular, Bold and Italic family does not qualify, a family with Thin, Light, Regular, Medium, Bold does.
- `FontApp` gets `collection: Option<Collection>` (`Arabic | English | Variable | ManyWeights`). Choosing a collection clears the chosen source and the other way round. `visible()` adds: collection test, and in list view only, `text_need = scripts_in(self.text)`; when non-zero, `family.scripts & text_need == text_need`.
- Counts for the four collections are computed when families change (after the scan and when folders are added), not every frame.
- Sidebar: a "Collections" section label and four rows in the same style as source rows. Above the list, while the text filter is on: a quiet one-line notice with the script names ("Showing fonts that support Arabic"); the preview text stays editable and clearing it removes the filter.
- Sketch:

```
Fonts                          Showing fonts that support Arabic
 All fonts      271            ┌────────────────────────────────┐
 System         239            │ Abd Elrady Pro · 2 styles      │
 User installed  23            │ مرحبا بالعالم                  │
Collections                    └────────────────────────────────┘
 Arabic          41
 English        230
 Variable         9
 Many weights    38
```

## Calls (the owner can overturn any of these)

- "English" means a font with Latin letters (so French or Spanish fonts count); the label stays "English" because that is the word the users use.
- "Arabic support" means the font has the letter ba (the existing Arabic test letter).
- "Many weights" counts distinct weights, not italics.
- Collections and sources are exclusive (picking one clears the other); the name search still applies on top of either.
- The filter hides fonts whenever the typed text needs a script the font lacks, even if only one letter is missing from a script the font otherwise has; rows still show "no glyphs for this text" for that case.

## What is left to the builder

The exact Unicode ranges per script, where the notice sits in the layout, the cached counts structure, and tests for `script_mask`, `scripts_in` and the family weight count.

## Verification

Unit tests for the mask and text-script functions, family aggregation and the `visible()` rules; the existing screenshot mode (`--text`, `--size`) for the notice and sidebar; `bench.ps1` against the budget; manual check that Arabic text hides non-Arabic fonts and clearing the text restores them.

## Amendments after the UX copy review (2026-11-23, owner chose "into the collections plan")

These override the wording above where they differ.

- Collection labels: **Arabic**, **English**, **Variable**, **4+ weights** (not "Many weights"). Each row has a tooltip ("Fonts that include Arabic letters", "Fonts that include Latin letters", "Fonts with adjustable axes such as weight or width", "Families with more than 3 different weights").
- The filter notice reads "Showing fonts that support Arabic" (mixed text: "Showing fonts that support Arabic and English") with a **Show all fonts** link that clears the preview text; there is no "x" button.
- Empty states always say why and what to do: search ("No fonts match "arial". Clear the search or pick another source."), collection ("No variable fonts found. Fonts with adjustable weight or width appear here."), text filter ("No fonts support Arabic. Show all fonts to see everything."), loading ("Scanning your fonts…").
- In the grid the preview box and size slider are hidden instead of dimmed (no "Used in the list view" tooltips).
- Copy pass on existing strings (banner without "Update required", "What's new" instead of "Release page", "Turned on by FontBase", the Arabic sentence in the details panel becomes English, clearer errors for missing or unloadable fonts, "Off. Turn on restores it as it was.", "Also in this file: ...").
- Not included (owner has not decided): a dismissable update banner, a fifth "Turned on" collection, deselect-on-second-click behaviour, the SmartScreen note on the site (belongs to the signing project).

## Amendments 2 (2026-11-23, after the sidebar review; the owner approved the structure)

These override anything above that differs.

- **Turned on** is a fifth filter, first in the sidebar after All fonts. It lists every family that is on for the user, whoever turned it on (Windows settings, FontCraft, FontBase or any font manager): exactly the families for which the app's existing `all_active(active, disabled, family)` is true. Windows' built-in system fonts are not "on" (nobody turned them on), so the list does not become a copy of All fonts. It refreshes right after a font is turned on or off, and it works the same way on macOS and Linux through the same `active` set.
- **Sidebar structure** (three groups with section labels, sentence case):
  - Library: All fonts, Turned on.
  - Collections: Arabic, English, Variable, 4+ weights.
  - Folders: System, User installed, From FontBase (Windows label; it replaces "Activated by FontBase", and it still holds fonts FontCraft turned off so they stay listed), every added folder, then the Add folder button.
- The middle of the sidebar scrolls (`ScrollArea`); the footer stays fixed: "{n} families · {m} styles" and the support card. At 720 x 480 nothing may be pushed out of view.
- Added folders can be removed: a small remove control (x) shown on hover of an added folder row, with the tooltip "Remove folder". Removing forgets the folder (saved list) and drops its fonts from the list; it never deletes files from disk and never turns fonts on or off.
- Picking any Library, Collection or Folder row is exclusive with the others (one active filter), as already decided for collections and sources.
