# Editor Shortcuts — Phase Checklist (CODE View)

Status: planning (to drive implementation and tests)

This document tracks keyboard navigation and indentation features for the CODE view. It is the authoritative TODO checklist to implement, test, and verify parity with TIC‑80 behavior. Items will be checked off as work lands.

## Scope (Phase 1)

- [x] Page Up / Page Down (plain)
  - Move caret up/down by visible line count (viewport height / 7), clamped.
  - Maintain a virtual column (desired x) across ragged lines.
  - Ensure caret remains visible by adjusting `scroll_line`.
  - Tests: basic motion, clamping, viewport re-centering/visibility.

- [x] Page Up / Page Down with Shift
  - Same movement but extend selection from anchor.
  - If no active selection, set anchor at caret before moving.
  - Tests: selection spans exactly N rows; correct range regardless of direction.

- [x] Ctrl+Home / Ctrl+End
  - Ctrl+Home → caret to (0,0); Ctrl+End → caret to (last line, `line_len(last)`).
  - Adjust viewport so caret is visible.
  - Tests: motion to start/end; clamping; viewport visibility.

- [x] Block Indent / Outdent (Tab / Shift+Tab)
  - When selection spans lines:
    - Tab → insert one leading space on each selected line.
    - Shift+Tab → remove one leading space when present.
  - Keep selection aligned to the same visual text after edits (adjust anchor/caret indices).
  - Undo/redo batching per operation (indent or outdent).
  - Tests: indent/outdent single- and multi-line selections; no-op when no leading space; undo/redo restores reliably.

## Integration Notes

- Platform modifiers
  - When this document says “Ctrl”, it means “Cmd” on macOS and “Ctrl” on Windows/Linux. The key handling treats either the Control or the “Logo” (Command) modifier as the shortcut modifier, consistent with existing copy/paste/undo/redo handling.
  - Shift variants behave the same on all platforms. PageUp/PageDown are used as delivered by the OS (on macOS keyboards these may be produced via Fn+Up/Down).

- Virtual column tracking
  - Add `desired_col` to remember horizontal intent across vertical/page moves; clamp to target line length but preserve for subsequent moves.

- Viewport & visibility
  - Reuse `ensure_visible()`; add helper to compute the page size from `area.h / 7`.
  - After any move, ensure caret lies within the viewport and adjust `scroll_line` as needed.

- Selection mechanics
  - Use `ensure_selection_anchor()` for Shift variants; otherwise `clear_selection()`.
  - When applying block edits, compute affected line range from selection start/end (convert char indices → line indices) regardless of anchor/caret order.

- Undo/redo
  - Use a single `EditOp` batch per multi-line indent/outdent for ergonomic undo.

- Key wiring (winit)
  - `VirtualKeyCode::PageUp` / `PageDown` (+Shift).
  - Ctrl/Cmd+`Home` / Ctrl/Cmd+`End`.
  - Tab / Shift+Tab: selection → block indent/outdent; no selection → keep current insert-space behavior.

## Out of Scope (Future Phases)

- Word navigation/delete (Ctrl/Alt+Left/Right, Ctrl+Backspace/Delete).
- Smart Home (toggle first non-whitespace vs col 0).
- Duplicate/delete line; move line up/down.
- Mouse selection (click/drag/double-click word/triple-click line).
- Find/Goto/Bookmarks/Outline/Run shortcuts.

---

Implementation will reference and update this checklist, checking items off as they land with tests.
