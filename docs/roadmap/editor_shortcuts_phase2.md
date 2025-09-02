# Editor Shortcuts — Phase 2 Plan (CODE View)

Status: planning (to drive implementation and tests)

This document is the Phase 2 checklist for common text editing features. It will be updated (checkboxes) as features and tests land.

## Scope (Phase 2)

- [x] Word Navigation / Edit
  - [ ] Ctrl/Alt+Left/Right: move caret by word boundaries.
  - [ ] Shift+Ctrl/Alt+Left/Right: extend selection by words.
  - [ ] Ctrl/Alt+Backspace/Delete: delete previous/next word.
  - Word boundary rules:
    - Treat sequences of letters/digits/underscore as a word.
    - Whitespace collapses; punctuation is its own word unit.
    - Tests: movement across mixed punctuation/whitespace; multi-line behavior; delete semantics consistent (no partial trailing spaces left).

- [x] Smart Line Bounds (Home toggle)
  - [ ] Home: toggle between first non-whitespace column and column 0.
  - [ ] Shift+Home: extend selection to the toggled bound.
  - Remember last toggle state per line until caret leaves the line.
  - Tests: lines with/without indentation; repeated Home presses; Shift variants.

- [x] Document Selection (to/from bounds)
  - [ ] Ctrl/Cmd+Shift+Home: select from caret to start of document.
  - [ ] Ctrl/Cmd+Shift+End: select from caret to end of document.
  - Tests: range correctness at arbitrary caret positions; anchor preserved.

## Integration Notes

- Platform modifiers
  - When this document says “Ctrl”, it means “Cmd” on macOS and “Ctrl” on Windows/Linux.

- Word boundary API
  - Add helpers to locate prev/next word boundary from a (line, col) pair; respect rope line limits.
  - Use these for both navigation and deletion to keep behavior consistent.

- Selection mechanics
  - Reuse `ensure_selection_anchor()` for Shift variants; otherwise `clear_selection()`.
  - Deleting by word with an active selection deletes the selection (standard editor behavior).

- Undo/redo
  - Word deletes are single operations (batch contiguous deletes if needed).

- Tests
  - Add unit tests for each key path, including edge cases (start/end of line, start/end of document, punctuation clusters, whitespace runs).

## Out of Scope (Future Phases)

- Duplicate/delete line; move line up/down.
- Comment toggling (Ctrl/Cmd+/).
- Mouse selection behaviors (click/drag/double-click/triple-click).
- Find/Goto/Replace.
- Column selection / multi‑cursor.

---

Implementation will reference and update this checklist, checking items off as they land with tests.
