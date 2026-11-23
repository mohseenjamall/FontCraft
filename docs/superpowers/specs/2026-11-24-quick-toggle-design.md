# Quick toggle and bulk turn on/off

Date: 2026-11-24. Status: approved by the owner in chat (approach A, both per-font and bulk).

## Intent and why
Turning a font on or off takes three steps today: click the row, wait for the details panel, press Turn on/off. People want one step, from the list or grid itself, and a way to turn on a whole filtered set (for example every Arabic font).

## Goals
- A switch on every list row and grid card. One click toggles that family. The click never opens the details panel.
- A bar above the list/grid that acts on the families currently shown: "Turn on N" and "Turn off N".
- Locked (system) fonts show a lock instead of a switch, with the reason on hover. Bulk skips them and says how many.

## Non-goals
No multi-select, no undo, no change to how fonts are turned on (same `toggle_in` and `turn_on_in`/`turn_off_in`), no keyboard shortcuts.

## Anti-goals
A switch that is easy to hit by accident while scrolling (it is small, away from the row's main click area) and a bulk button that surprises (it states the exact count, and asks once above 50 families).

## Approaches
1. Switch on each row/card plus a bulk bar that follows the current filter (chosen). Reuses the filters; no new state beyond a pending confirmation.
2. Checkbox multi-select then a button: more flexible, adds steps and selection state. Not now.

## Design (the owner's calls in chat, mine marked)
- Switch: 34x18 pill. On = cobalt track, white knob right. Off = hairline track, knob left. List rows: always shown, right side, replacing the "On" dot. Grid cards: always when on, on hover or selected when off (my call: less noise in a 100-card page).
- Locked: a "Locked" text in muted ink with the lock reason as tooltip; no switch.
- Bulk bar, top of the content area under the filter notice: "{shown} families · {on} on" then "Turn on {n}" (primary) when n > 0 and "Turn off {m}" (ghost) when m > 0. Locked count appended: "· {k} locked".
- Above 50 families a bulk click shows an inline "Turn on 120 fonts? Turn off restores them." with "Turn on" and "Cancel". Any filter/search change cancels it (my call).
- After a bulk run, a one-line note "Turned on 72 families." (click to dismiss); errors still use the red notice.
- One call into `turn_on_in`/`turn_off_in` for all files, so Windows broadcasts the font change once.
- The "Turned on" empty text now says "Turn a font on with its switch."

## Left to the builder
Exact pixel spacing; tests for the bulk selection logic (skips locked, skips already-in-state, dedups shared files).
