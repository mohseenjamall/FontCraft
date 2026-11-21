# Font Activation (Turn Fonts On/Off) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a person turn fonts from their own folders on and off for all Windows apps, from the FontCraft details panel.

**Architecture:** A new `activate.rs` module owns every Win32/registry call. Activation = `AddFontResourceW(path)` + a marker value in the current user's font registry key (so it survives logoff) + a `WM_FONTCHANGE` broadcast; deactivation reverses all three. The UI keeps a `HashSet<PathBuf>` of active files, loaded at startup from the registry, and toggles it through `activate.rs`. Activation is per **file** (a `.ttc` holds several families), never per face.

**Tech Stack:** Rust 2024, eframe/egui 0.37, `winreg` 0.56 (already present), `windows-sys` (new, Windows-only).

**Spec:** `MECHANICS.md` section "Native code (decompiled with Ghidra)" (FontBase's `registerFonts` / `unregisterFonts` behaviour). Decisions made here, not in that spec:
- FontCraft writes its **own** marker suffix `" (FontCraft)"` and never edits FontBase's `" (FontBase)"` values, so both apps can coexist.
- Fonts FontBase activated show as active but their toggle is disabled.
- Only fonts from folders the person added (`source >= fixed_sources`) can be toggled. System and "User installed" fonts are already installed.

## Global Constraints

- Registry key: `HKCU\Software\Microsoft\Windows NT\CurrentVersion\Fonts`, value data = full file path, type `REG_SZ`.
- Marker value name: `"<file stem> <8 lowercase hex> (FontCraft)"` where the hex is FNV-1a 32-bit of the lowercased full path.
- Broadcast: `PostMessageW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0)` after every activate/deactivate batch.
- Windows only: everything new is `#[cfg(windows)]`, with a no-op fallback so `cargo build` still works elsewhere.
- No new files over 400 lines; follow existing style (`//!` module doc, no comments restating code).
- Release build stays statically linked (`.cargo/config.toml` already sets the CRT flag); do not add a dependency that needs a DLL.

## Review Focus

- Non-ASCII path (Arabic/CJK folder or file name): must activate and deactivate correctly (wide strings, no lossy conversion).
- File deleted/moved while active: startup must not list it as active and deactivate must still remove its marker without error.
- `.ttc` shared by several families: toggling one family changes all families in that file; the UI must say so.
- Two files with the same name in different folders: markers must not collide.
- Registry write fails (no permission / key missing): return `Err(String)`, show it in the UI, leave state unchanged, never panic.
- Toggle twice quickly / already active: idempotent, no duplicate values.

---

## File Structure

| File | Responsibility |
|---|---|
| `Cargo.toml` | add `windows-sys` (Windows only) |
| `src/activate.rs` (new) | marker naming, registry read/write, Win32 add/remove, broadcast |
| `src/main.rs` | `mod activate;` |
| `src/scan.rs` | `fontbase_activated()` unchanged; reused as the FontBase-owned set |
| `src/app.rs` | active set, toggle button + status in details panel, "On" badge in list row |
| `README.md`, `MECHANICS.md` | document the feature and the own-suffix decision |

---

### Task 1: Marker naming and registry core

**Files:**
- Modify: `Cargo.toml`, `src/main.rs`
- Create: `src/activate.rs`

**Interfaces:**
- Produces:
  - `pub const FONTS_KEY: &str` = `r"Software\Microsoft\Windows NT\CurrentVersion\Fonts"`
  - `pub fn marker(path: &Path) -> String`
  - `fn write_marker(key: &str, path: &Path) -> Result<(), String>`
  - `fn remove_marker(key: &str, path: &Path) -> Result<(), String>`
  - `pub fn read_active(key: &str) -> HashSet<PathBuf>` (values ending `" (FontCraft)"` **or** `" (FontBase)\n"`, only paths where `is_file()`)
  - `fn is_owned(key: &str, path: &Path) -> bool` (true only for a FontCraft marker)

- [ ] **Step 1: Add dependency and module**

In `Cargo.toml` under `[target."cfg(windows)".dependencies]` add
`windows-sys = { version = "0.61", features = ["Win32_Graphics_Gdi", "Win32_UI_WindowsAndMessaging", "Win32_Foundation"] }`.
Add `mod activate;` to `src/main.rs`. Create `src/activate.rs` with a `//!` doc line and `#[cfg(test)] mod tests`.

- [ ] **Step 2: Write the failing tests** (all use a scratch key `Software\FontCraftTest\Fonts`, deleted at the end of each test)

```rust
fn marker_is_stable_and_case_insensitive()   // marker(Path::new(r"C:\A\Foo.ttf")) == marker(Path::new(r"c:\a\foo.ttf"))
fn marker_differs_for_same_stem_in_two_folders() // C:\A\x.ttf vs C:\B\x.ttf
fn marker_has_expected_shape()               // starts with "Foo ", ends with " (FontCraft)", 8 hex digits between
fn write_then_read_then_remove_roundtrip()   // temp file on disk; write_marker -> read_active contains it -> remove_marker -> read_active empty
fn read_active_skips_missing_files()         // marker pointing at a deleted file is not returned
fn read_active_includes_fontbase_values()    // set value "Bar (FontBase)\n" -> path appears
fn is_owned_false_for_fontbase_value()
fn write_marker_twice_is_idempotent()        // still exactly one value
fn unicode_path_roundtrip()                  // temp file named "خط-字.ttf"
```

- [ ] **Step 3: Run to verify they fail**

Run: `cargo test activate:: -- --test-threads=1`
Expected: compile error / FAIL ("function not defined").

- [ ] **Step 4: Implement the functions above in `src/activate.rs`**

Use `winreg::RegKey::predef(HKEY_CURRENT_USER).create_subkey(key)` for writes, `open_subkey_with_flags(key, KEY_READ)` for reads. FNV-1a 32 over the lowercased `path.to_string_lossy()` bytes is only the *name* hash, so lossy is fine there; the registry *data* must be written from the `OsStr` (winreg `RegValue` from `OsStr`), not lossy. `remove_marker` treats "value not found" as success. `read_active` returns an empty set if the key is missing.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test activate:: -- --test-threads=1`
Expected: 9 passed.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs src/activate.rs
git commit -m "feat: registry markers for activated fonts"
```

---

### Task 2: Activate and deactivate (Win32)

**Files:**
- Modify: `src/activate.rs`

**Interfaces:**
- Consumes: Task 1 `marker`, `write_marker`, `remove_marker`, `is_owned`, `FONTS_KEY`.
- Produces:
  - `pub fn activate(paths: &[PathBuf]) -> Result<usize, String>` (returns how many files were newly activated)
  - `pub fn deactivate(paths: &[PathBuf]) -> Result<usize, String>`
  - Both use `FONTS_KEY`; internal `fn activate_in(key: &str, paths: &[PathBuf])` / `fn deactivate_in(key: &str, paths: &[PathBuf])` take the key so tests use the scratch key.

- [ ] **Step 1: Write the failing tests** (scratch key; each copies `C:\Windows\Fonts\arial.ttf` into a temp dir and removes everything at the end)

```rust
fn activate_makes_font_active_and_loads_it()      // activate_in -> Ok(1); read_active contains path; AddFontResourceW(path) side effect verified by a second add returning >= 1
fn activate_twice_reports_zero_new()              // second call Ok(0), still one value
fn deactivate_removes_marker_and_resource()       // after deactivate_in: read_active empty, Ok(1)
fn deactivate_missing_file_still_removes_marker() // delete file first, then deactivate_in -> Ok, marker gone
fn deactivate_ignores_fontbase_values()           // a "(FontBase)\n" value for that path survives deactivate_in
fn activate_nonexistent_file_errors()             // Err(msg) mentioning the path, no marker written
fn activate_unicode_path()                        // same as the ASCII one with an Arabic file name
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test activate:: -- --test-threads=1`
Expected: FAIL (functions not defined).

- [ ] **Step 3: Implement**

- `fn wide(p: &Path) -> Vec<u16>` using `std::os::windows::ffi::OsStrExt`, NUL-terminated.
- `activate_in`: for each path not already owned: require `is_file()` else return `Err(format!("{}: file not found", path.display()))` before touching anything; `AddFontResourceW` returning `0` → `Err`; then `write_marker`. Count newly activated. After the loop, if count > 0, `broadcast()`.
- `deactivate_in`: for each owned path: `RemoveFontResourceW` (ignore a `0` return because the file may already be gone), then `remove_marker`. Broadcast if anything changed.
- `fn broadcast()`: `PostMessageW(HWND_BROADCAST, WM_FONTCHANGE, 0, 0)`.
- `#[cfg(not(windows))]` stubs for `activate`/`deactivate`/`read_active` returning `Err("Windows only")` / empty set.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test activate:: -- --test-threads=1`
Expected: 16 passed.

- [ ] **Step 5: Commit**

```bash
git add src/activate.rs
git commit -m "feat: activate and deactivate font files"
```

---

### Task 3: Toggle in the UI

**Files:**
- Modify: `src/app.rs` (struct `FontApp` ~39-55, `new` ~58-86, `row` ~297-324, `details` ~326-367)

**Interfaces:**
- Consumes: `activate::read_active(FONTS_KEY) -> HashSet<PathBuf>`, `activate::activate/deactivate`, `scan::fontbase_activated() -> Vec<PathBuf>`.
- Produces:
  - `FontApp.active: HashSet<PathBuf>`, `FontApp.notice: Option<String>`
  - `impl Family { fn files(&self) -> Vec<PathBuf> }` (distinct file paths, sorted)
  - `impl FontApp { fn can_toggle(&self, fam: &Family) -> bool; fn toggle(&mut self, name: &str) }`
  - `can_toggle` is true only when every face has `source >= self.fixed_sources`.

- [ ] **Step 1: Write the failing tests** in `app.rs` `tests` (they exercise pure logic, no registry)

```rust
fn family_files_are_distinct_and_sorted()      // 3 faces, 2 distinct paths -> vec of 2
fn can_toggle_false_for_built_in_sources()     // face.source = 0 with fixed_sources = 3
fn can_toggle_true_for_added_folder()          // face.source = 3 with fixed_sources = 3
fn is_active_requires_all_files_active()       // helper fn all_active(&HashSet<PathBuf>, &Family) -> bool
```

(Build a `FontApp` for these through a small `FontApp::for_test(fixed_sources: usize)` constructor that skips the scan thread; add it under `#[cfg(test)]`.)

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test app:: -- --test-threads=1`
Expected: compile error / FAIL.

- [ ] **Step 3: Implement**

- `new`: `active = activate::read_active(activate::FONTS_KEY)`; `notice = None`.
- `toggle(name)`: find family; `files = fam.files()`; if `all_active` call `deactivate(&files)` else `activate(&files)`. On `Ok` re-read `self.active = read_active(...)`; on `Err(e)` set `self.notice = Some(e)`. Never panic.
- `row`: if `all_active`, draw a small filled "ON" pill (colour `ACTION`) at the right end of the meta line.
- `details`: under the path label add a button "Turn on" / "Turn off" (`ACTION` fill when turning on). Disabled with hover text "Built-in fonts are always on" when `!can_toggle`, and "Turned on by FontBase" when the files are in `fontbase_activated()`. If the family's files hold faces of other families, show a gray line "This file also holds: <names>" (compute from `self.faces` by path). Show `self.notice` in red under the button with a dismiss on click.

- [ ] **Step 4: Run tests and build**

Run: `cargo test` then `cargo build --release`
Expected: all tests pass, build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: turn font families on and off from the details panel"
```

---

### Task 4: End-to-end check and docs

**Files:**
- Modify: `README.md`, `MECHANICS.md`
- Create: `docs/plans/2026-11-21-font-activation.checklist.md` (optional: paste the manual results)

- [ ] **Step 1: Manual run on a copy of a font**

Copy a `.ttf` into `C:\Temp\fc-test\`, run `target\release\fontcraft.exe`, **+ Add folder** that folder, select the family, click **Turn on**.
Expected: "ON" pill appears; the font shows in Word/Notepad's font list without restarting them; `reg query "HKCU\Software\Microsoft\Windows NT\CurrentVersion\Fonts" | findstr FontCraft` shows one value.

- [ ] **Step 2: Persistence**

Close FontCraft, reopen it: the pill is still there. Sign out and back in (or restart): the font is still available in other apps.

- [ ] **Step 3: Turn off**

Click **Turn off**. Expected: pill gone; value gone; font disappears from Word's list after reopening Word.

- [ ] **Step 4: Edge cases from Review Focus**

Repeat steps 1-3 with a folder named in Arabic; with a `.ttc` (confirm the "also holds" line); with a file deleted while on (restart FontCraft: not listed active; no error).

- [ ] **Step 5: Docs**

README "Not in version 1": remove "Turning fonts on and off". Add a "Turn fonts on and off" bullet under Features. MECHANICS.md: add one row "FontCraft's own marker `<stem> <hash> (FontCraft)`, never touches `(FontBase)` values".

- [ ] **Step 6: Final checks and commit**

Run: `cargo test` and `cargo build --release`
Expected: all pass.

```bash
git add README.md MECHANICS.md docs
git commit -m "docs: font activation"
```

---

## Not in this plan (YAGNI)

Collections/tags, "activate all fonts in a folder" and bulk activation progress (activation runs on the UI thread; shortcut: fine for one family, move to a worker thread if a family ever has more than ~50 files), an "Active only" sidebar filter, auto-deactivate on exit.

## Self-review

- Spec coverage: AddFontResourceW + registry + broadcast (Tasks 1-2), deactivate = Remove + delete value (Task 2), UI (Task 3), persistence verified (Task 4).
- Type consistency: `marker/write_marker/remove_marker/read_active/is_owned/activate/deactivate/activate_in/deactivate_in/FONTS_KEY` are named the same in every task.
- Review Focus: unicode (T1, T2, T4), missing file (T1, T2, T4), `.ttc` (T3 line, T4), same-stem collision (T1), registry error (T2, T3 notice), idempotence (T1, T2).
