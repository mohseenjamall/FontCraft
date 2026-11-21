# Font preview — rules taken from FontBase 2026.7.4

FontBase.exe is the stock Electron (Chromium) shell; the feature lives in the JavaScript bundle
inside `resources/app.asar` (evidence ev_fefb7ffe…, extracted as ev_0cffe4e6…). The native
program holds no FontBase-specific logic, so the numbers below come from the bundle, not from
decompiled machine code.

## Native code (decompiled with Ghidra)

FontBase's own native code is the add-on `fontsession.node` in the app package
(`dist/84768dbecfaf68a2bd3970c1a827b0a7.node`, built from `fontbase-app/app/fontsession/src/fontsession.cc`).
It exports two functions, `registerFonts` and `unregisterFonts` (names at 0x180017c68 / 0x180017c78).

| Rule | Value | Function |
|---|---|---|
| Activate a font | `AddFontResourceW(path)` for every `{name: path}` entry | `registerFonts`, 0x180001200 |
| Remember an activated font | registry key `HKCU\Software\Microsoft\Windows NT\CurrentVersion\Fonts`, value name `"<name> (FontBase)\n"`, REG_SZ data = full file path (any folder) | 0x180001200 |
| Tell other apps | `PostMessage(HWND_BROADCAST 0xFFFF, WM_FONTCHANGE 0x1D)` after (un)registering | 0x180001200, 0x1800017f0 |
| Deactivate | `RemoveFontResourceW(path)`, then delete value `"<name> (FontBase)\n"` from the same key | `unregisterFonts`, 0x1800017f0 |

Used in the rebuild: the list reads that registry key and shows every value containing
" (FontBase)" as the source **Activated by FontBase**, so fonts FontBase turned on from other
folders appear too (`scan::fontbase_activated`).

### FontCraft's own activation (`src/activate.rs`)

Same three steps as FontBase's add-on, with its own marker so the two apps can share the key. It
can also turn off fonts installed for the user and fonts FontBase turned on, saving their values so
they come back exactly:

| Rule | Value | Source |
|---|---|---|
| Turn on | file must exist, then `AddFontResourceW(path)`, then write the marker; undo the load if the write fails | follows 0x180001200 |
| Marker | value `"<file stem> <8-hex FNV-1a 32 of the lowercased path> (FontCraft)"` = REG_SZ full path, in the same `HKCU\…\Fonts` key | FontCraft decision (hash keeps same-named files in two folders apart) |
| Turn off | `RemoveFontResourceW(path)` (a 0 return is fine: the file may be gone), then delete the marker | follows 0x1800017f0 |
| Broadcast | `PostMessageW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0)` once per batch that changed something | 0x180001200 / 0x1800017f0 |
| What counts as on | every file of the family has some value in `HKCU\…\Fonts` (any name: `X (TrueType)`, `(FontBase)`, `(FontCraft)`) whose data is that file and the file exists; off-and-saved when its values are in the saved list | FontCraft decision |
| Who may toggle | every family with no file directly in `%WINDIR%\Fonts`; those are system fonts (HKLM, all users) and show "System font: needs administrator" | FontCraft decision |
| Values of a file | every `HKCU\…\Fonts` value whose REG_SZ / REG_EXPAND_SZ data is the file, compared case-insensitively; relative names resolve against `%LOCALAPPDATA%\Microsoft\Windows\Fonts` | `activate::values_for`, `disabled::data_path` |
| Turn off (installed / FontBase) | save each value (name, type, raw bytes) to `%APPDATA%\FontCraft\disabled.tsv` first, then `RemoveFontResourceW`, delete the values, `WM_FONTCHANGE`. A `(FontCraft)` marker is just removed, not saved | `activate::turn_off_in` |
| Turn on (saved) | `AddFontResourceW`, write every saved value back byte for byte under its original name, drop them from the saved list, `WM_FONTCHANGE`; with nothing saved, write a `(FontCraft)` marker | `activate::turn_on_in` |
| Saved list | `disabled.tsv`, one value per line: `type<TAB>hex(name)<TAB>hex(bytes)`; missing = empty; unreadable lines are kept as they are on rewrite; written to a temp file then renamed | `src/disabled.rs` |
| Safety | FontCraft never deletes a font file and never writes HKLM; system fonts are refused with an error before anything is touched | `activate::is_system_font` |
| FontBase | a turned-off FontBase font stays listed (its saved file is scanned); FontBase may still show it as on until opened there, since FontBase keeps its own list | FontCraft decision |
| Unit | per file: a `.ttc` holding several families turns them all on or off together | Windows loads whole files |

FontBase.exe (214 MB) is the stock Electron/Chromium shell. Searching its strings for font
settings finds only Chromium's web-preferences reader (`defaultFontSize`, `minimumFontSize`,
referenced from sub_1403ba090); two attempts to decompile that function stopped the session
before returning, so it stays unread. It is the browser's page default, not FontBase's slider.

### Update notice (`src/update.rs`)

| Rule | Value | Source |
|---|---|---|
| Latest version | the tag in GitHub's redirect from `https://github.com/mohseenjamall/FontCraft/releases/latest` to `…/releases/tag/<tag>` (no API, JSON or token) | FontCraft decision |
| When | once per start, on a background thread, `curl.exe -s --max-time 10`, no console window | FontCraft decision |
| Newer | `vMAJOR.MINOR.PATCH` (leading `v` optional, missing parts 0, `-`/`+` suffix ignored) strictly above `CARGO_PKG_VERSION` | `update::is_newer` |
| Trusted link | the button only opens URLs starting `https://github.com/mohseenjamall/FontCraft/releases/` with no spaces, quotes, backslashes or `..` | `update::trusted` |
| Banner | top bar in the accent colour, cannot be closed: "Update required: FontCraft <version> is available. Please update now." + Download update | `app::update_banner` |
| No answer | offline, blocked, no curl, private repo (404) → no banner, no error | `update::check` |
| Checks | `--screenshot` runs skip the network unless `FONTCRAFT_LATEST=<tag>` stands in for GitHub's answer | `main.rs` |

## JavaScript bundle

| Rule | Value | Where it was read |
|---|---|---|
| Preview size slider | min 6, max 180, step 2 px | `dist/app.js`, size slider bound to the `fontSize` setting |
| Default preview size | 42 px | `dist/app.js`, preview settings default `fontSize:42` |
| Waterfall sizes (detail view) | 72, 48, 24, 14 px | `dist/app.js`, waterfall component `size:72/48/24/14` |
| Default sample per writing system | first script whose test letter the font has: Latin `A`, Chinese `我`, Japanese `私`, Devanagari `अ`, Arabic `ب`, Cyrillic `и`, Greek `ξ`, Khmer `ក`, Lao `ກ`, Thai `ข`, Tamil `அ`, Myanmar `က`, Bengali `অ`, Hebrew `א`, Armenian `Ա`, Georgian `Ⴀ`; Latin when none | `dist/talker.js`, script table + `charToGlyph(test).index != 0` |
| Font name fields used | typographic family (name id 16) else family (1); typographic subfamily (17) else subfamily (2) | `dist/parser.js` name-table field list |
| Header background | #111111 | `dist/style.css` `--color-back-header` (default theme) |
| Sidebar background | #181818 | `dist/style.css` `--color-back-side` |
| Main background | #FAFAFA, text #222222 | `dist/style.css` `--color-back-main`, `--color-text-main` |
| Accent (action) | #5200EA | `dist/style.css` `--color-action` |
| Waterfall line height | 1.35 | `dist/style.css` `.lines_*` |

Estimated (looked for, not found as plain values; strings in `app.js` are obfuscated):
- Exact default sentence when the person types nothing: FontBase's own text is obfuscated; the
  Latin sentence from talker.js is used instead.
- List row height: font size × 1.35 + 34 px.

Left out of this version: collections and tags, Google Fonts, glyph map, OpenType feature toggles, variable-font
sliders, cloud sync.
