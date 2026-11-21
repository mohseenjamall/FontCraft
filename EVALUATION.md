# FontBase vs FontCraft — Speed Test and Evaluation

*Tested 21 November 2026 on the same computer, one after the other: AMD Ryzen 7 5800X (8 cores),
32 GB RAM, Windows 11 Pro. Both apps saw the same fonts (about 270 families / 540 styles).*

| | FontBase (original) | FontCraft (new) |
|---|---|---|
| Version | 2026.7.4 | 1.0.0 |
| Built with | Electron (a full Chromium web browser + JavaScript/React) | Rust + egui (native, no browser inside) |

## 1. Speed test results

Each app was started 3 times from fully closed. The table shows the middle (median) result.
Memory and CPU were measured 20 seconds after start-up, with the app's window open.

| Measure | FontBase | FontCraft | FontCraft is… |
|---|---|---|---|
| Start-up until the app is ready and quiet ¹ | **6.21 s** (4.28 – 7.26) | **1.46 s** (0.95 – 2.32) | **~4× faster** |
| Window appears on screen | starts hidden in the tray ² | **0.28 s** | — |
| Open window from tray (already running) | 0.26 s | n/a (no tray) | — |
| Every font listed and drawn in the window | not measurable from outside ³ | **≤ 1.92 s** (1.92 – 1.93) ⁴ | — |
| Memory in use (working set) | **784 MB** (765 – 865) | **104 MB** (101 – 104) | **7.5× less** |
| Memory reserved for itself (private) | **758 MB** | **110 MB** | **6.9× less** |
| CPU time used while starting | **9.95 s** (5.62 – 18.11) | **0.81 s** (0.77 – 0.86) | **~12× less** |
| Programs running (processes) | **5** | **1** | 5× fewer |
| Threads | **182** | **18** | 10× fewer |
| Space on disk | **438.3 MB** | **6.3 MB** | **70× smaller** |

Also measured before the test, with FontBase already running in the tray since it was started
earlier that morning: 5 processes, **683 MB** working set, **570 MB** private memory.

¹ *Ready and quiet*: all of the app's processes together use under 3% of one CPU core for one full
second. This is the fairest common finish line, because FontBase starts hidden.
² FontBase is set to start in the system tray, so no window appears until you open it.
³ FontBase draws its font list inside its built-in browser, which can't be timed from outside
without changing the app.
⁴ An upper limit: the test always waits at least 120 screen refreshes (about 2 s at 60 Hz)
before taking its picture, so the real time may be shorter.

### Raw results

| App | Run | Ready (s) | Window (s) | Tray → window (s) | Processes | Working set (MB) | Private (MB) | CPU (s) | Threads |
|---|---|---|---|---|---|---|---|---|---|
| FontBase | 1 | 4.28 | — | 0.24 | 5 | 784 | 701 | 9.95 | 185 |
| FontBase | 2 | 7.26 | — | 0.26 | 5 | 765 | 758 | 5.62 | 182 |
| FontBase | 3 | 6.21 | — | 0.26 | 5 | 865 | 765 | 18.11 | 180 |
| FontCraft | 1 | 1.46 | 0.06 | — | 1 | 104 | 110 | 0.81 | 18 |
| FontCraft | 2 | 2.32 | 0.37 | — | 1 | 104 | 112 | 0.86 | 18 |
| FontCraft | 3 | 0.95 | 0.28 | — | 1 | 101 | 107 | 0.77 | 18 |

FontCraft "every font listed and drawn": 1.93 s, 1.92 s, 1.92 s.

The script that produced these numbers is `bench.ps1`, and its full output is in
`bench-results.json`. Run it again with
`powershell -ExecutionPolicy Bypass -File bench.ps1`. It closes FontBase while testing and starts it
again at the end.

### Why FontCraft is lighter

- **No browser inside.** FontBase carries a whole Chromium browser (the 214 MB main program plus
  helper processes for drawing, the GPU and background work). FontCraft is one small program that
  draws straight to the screen.
- **Loads only what you see.** FontCraft reads every font's name on 8 threads at once, but loads a
  font's full file only when its row scrolls into view.
- **One process.** No messages passing between a main process, a renderer and background workers.

## 2. Features compared

| Feature | FontBase | FontCraft |
|---|---|---|
| List all installed fonts, grouped by family | ✅ | ✅ |
| Preview each font in its own typeface | ✅ | ✅ |
| Type your own preview text | ✅ | ✅ |
| Preview size slider (6 – 180 px, starts at 42) | ✅ | ✅ same range |
| Right sample for non-Latin fonts (16 writing systems) | ✅ | ✅ same rule and order |
| Family detail: every style + waterfall at 72 / 48 / 24 / 14 px | ✅ | ✅ |
| Search by name | ✅ | ✅ |
| Add your own font folders (remembered) | ✅ | ✅ |
| Shows fonts FontBase has activated | ✅ | ✅ reads FontBase's own registry entries |
| Turn fonts on / off without installing | ✅ | ❌ not in v1 |
| Collections and tags | ✅ | ❌ |
| Filters by weight, contrast, proportion, x-height, duplicates | ✅ | ❌ |
| Google Fonts and online font sources | ? not confirmed in its files ⁵ | ❌ |
| Character (glyph) map | ✅ | ❌ |
| OpenType feature and variable-font controls | ✅ | ❌ |
| Text colour, background, alignment, print | ✅ | ❌ |
| Dark / light themes | ✅ several | Fixed: dark header and sidebar, light list (FontBase's default look) |
| Account sign-in, sync, auto-update, start with Windows, tray icon | ✅ | ❌ |
| Works offline, no account | Partly (sign-in features need an account) | ✅ fully |

⁵ FontBase's on-screen text is scrambled inside its files, so this row could not be confirmed
either way. Every other ✅ for FontBase was found in its files (settings, IPC channels or code).

## 3. Evaluation

### FontCraft — strengths
- About 4× faster to start, using 7× less memory, 12× less CPU and 70× less disk space.
- Ready to use as soon as it opens; the font list fills in within about 2 seconds.
- Simple: one window and one job, browsing and previewing fonts, with nothing to sign into.
- Shows the same fonts as FontBase, including the ones FontBase activated from other folders.

### FontCraft — weaknesses
- Covers only the main feature: it can't turn fonts on or off, organise them into collections, or
  show a character map.
- A few fonts with unusual letter coverage show empty boxes instead of letters. FontBase picks its
  sample by the same rule, so it may do the same.
- Windows only for now, and it has no installer yet (you run the program file directly).

### FontBase — strengths
- A complete font manager: activation, collections, tags, smart filters, glyph map,
  OpenType and variable-font controls, printing, themes and sync.
- Polished and mature, with automatic updates.

### FontBase — weaknesses
- Heavy for what it shows: about 0.7 – 0.9 GB of memory and 5 background processes, even while
  sitting in the tray.
- Takes 4 – 7 seconds to settle after starting.
- About 440 MB on disk, mostly the browser engine it carries.

### Verdict
For quickly browsing and comparing fonts, **FontCraft is the better tool**: it is ready about 4×
sooner and uses about 7× less memory. For managing a large library (turning fonts on and off,
collections, filters), **FontBase is still needed**. The best next step for FontCraft would be
turning fonts on and off, which FontBase's own native code does with three Windows calls (see
`MECHANICS.md`).

## 4. Limits of this test
- Run on one computer, 3 times per app. Results will differ on slower machines and with larger
  font libraries.
- "Ready" means the app stopped using the CPU. It is a common finish line for both apps, not the
  moment the font list became visible.
- FontBase's time to draw its font list was not measured (see ³).
- FontBase's memory includes the work it does in the background, such as checking for updates and
  watching font folders. FontCraft does none of that.
