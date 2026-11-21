# Turn user-installed and FontBase fonts on and off

**Goal:** the Turn on / Turn off button also works for fonts installed for the current user and for
fonts FontBase turned on. System fonts stay locked. No font file is ever deleted; HKLM is never written.

## Rules
- **System font** = file directly in `%WINDIR%\Fonts`. Never toggled; the panel says
  "System font: needs administrator". `turn_on/turn_off` return `Err` before touching anything.
- **Which registry values belong to a file:** every value in `HKCU\…\Fonts` whose data (REG_SZ /
  REG_EXPAND_SZ, decoded without loss; relative names resolved against
  `%LOCALAPPDATA%\Microsoft\Windows\Fonts`) is that file, compared case-insensitively. Any name:
  `Arial (TrueType)`, `<name> (FontBase)\n`, `<stem> <hash> (FontCraft)`.
- **Turn off:** save every non-FontCraft value of the file (name, type, raw bytes) to
  `%APPDATA%\FontCraft\disabled.tsv` *first*, then `RemoveFontResourceW`, then delete the values,
  then `WM_FONTCHANGE`. A FontCraft marker is simply removed (as before). Nothing to turn off = no-op.
- **Turn on:** if the file has saved values, write them back byte-for-byte under the same name,
  `AddFontResourceW`, drop them from the saved list. Otherwise (added-folder font) write a FontCraft
  marker as before. Then `WM_FONTCHANGE`.
- **Shown state:** ON when every file of the family has a value pointing at an existing file; OFF
  when any file has saved values. Computed from the registry and the saved list each time.
- **Saved list** (`disabled.tsv`, one value per line: `type<TAB>hex(name)<TAB>hex(bytes)`): a missing
  file is empty; a line that can't be read is kept as-is when the file is rewritten, so nothing is
  ever lost. Written to a temp file and renamed.
- **Who may toggle:** every family with no face in `%WINDIR%\Fonts`.
- FontBase may still list a font as on after FontCraft turns it off; the panel says so.

## Deviation from the request
The saved list is `disabled.tsv`, not `disabled.json`: it's a simple line format (no new
dependency), and a `.json` name would promise JSON.

## Tests (scratch key `Software\FontCraftTest\…`, scratch saved list)
user-installed off/on round trip byte-identical; FontBase value round trip; off twice keeps the
saved value once; off with the file deleted still saves; missing or corrupt saved list; Arabic path;
system font returns Err and leaves registry and saved list untouched; `can_toggle` for system /
user-installed / FontBase / added folder; live `#[ignore]` test on the real key that cleans up.
