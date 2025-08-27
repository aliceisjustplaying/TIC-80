# Livecoding Editor Plan (TIC‑80 Vibes)

This plan describes a minimal, purpose‑built editor focused on livecoding with TIC‑80’s look and feel. Scope is limited to a Code view (delivered first) and a Console view (delivered after CODE; scope may be reduced), tightly integrated with the runtime and preserving .tic semantics. Sprite/Map/SFX/Music editors are out‑of‑scope for this rewrite.

## Scope and Goals
- Sequencing: CODE first; CONSOLE after. Console scope may be reduced based on needs.
- Views: CODE, CONSOLE (preview is the running framebuffer as today).
- Parity: Preserve .tic cart semantics. Replace only the code section on save; all other sections remain byte‑for‑byte identical.
- Vibes: Render a pixel‑perfect TIC‑80 UI skin in the 240×136 framebuffer and present via integer scaling.
- Livecoding: Hot‑reload on save; keep last‑good runner if the new code has errors.

## Architecture
- Rendering: Draw the full UI (chrome, tabs, editor, console, status) into our 240×136 palette framebuffer using existing gfx primitives; present with `winit + pixels` integer scaling.
- Input: Read keyboard/mouse from `winit` and feed a simple UI layer (no external widget toolkit).
- Text engine: Rope‑backed buffer for code (e.g., ropey) to keep edits fast and memory stable.
- Runtime coupling: Save swaps LuaRunner safely; error messages go to the console while keeping the last‑good runner active.
- Cart I/O: Code‑only patching on save; everything else preserved exactly.

## UI Skin
- Top bar: TIC‑80‑style chrome, CODE/CONSOLE tabs, RUN/STOP/RESET buttons, FPS/ms status.
- CODE: Monospace text area with gutter (line numbers), caret, selection, scrolling, basic syntax coloring.
- CONSOLE: Scrolling list of trace() lines and errors; timestamps; clear button.
- Bottom status: File name, cursor position, modified flag.

## Phases and Detailed TODOs (CODE first, then CONSOLE)

Phase 0 — Shell + Layout
- [ ] Create `docs/` UI skin reference (palette, font, margins, tab geometry) (optional diagrams).
- [x] Add a minimal UI state model (active tab, focus, scroll positions).
- [x] Render top chrome, tabs, and placeholder panels into the framebuffer.
- [x] Wire tab switching and button hit‑testing (rect hit tests in 240×136 space).
- [x] Integrate integer scaling to window and mouse coordinate unprojection.

Phase 1 — Text Engine + Editor Basics
- [x] Integrate a rope‑backed buffer (ropey) and load cart code into it.
- [x] Caret movement (arrows) and auto-scroll; Home/End pending.
- [ ] Insert/delete/backspace, newlines.
- [ ] Selection (shift+arrows), clipboard (Ctrl/Cmd+C/V/X), undo/redo (local stack for code buffer).
- [x] Horizontal/vertical scrolling; viewport mapping from text rows to framebuffer pixels.
- [x] Draw gutter (line numbers).

Phase 2 — Syntax + UX polish (minimal)
- [ ] Lightweight Lua colorizer (keywords, comments, strings, numbers) with palette colors.
- [ ] Find/replace panel (Ctrl/Cmd+F) with next/prev navigation.
- [ ] Adjustable font scale within 8×8 multiples (e.g., 1×/2×) while preserving 240×136 layout.

Phase 3 — Console Pane (may be reduced)
- [ ] Console ring buffer model with timestamps and color tags.
- [ ] Route trace() and runtime/loader errors to the console.
- [ ] Clear button and scroll behavior; persist scroll at bottom on new lines.

Phase 4 — Hot Reload + Cart I/O
- [ ] Code‑only .tic write: patch code section and save; verify other sections preserved.
- [ ] Hot reload: on save, compile+swap LuaRunner; on error, push message to console and keep last‑good runner.
- [ ] Menu/buttons: Run/Stop/Reset wired to runtime control.

Phase 5 — Tests + Docs
- [ ] Headless test: .tic round‑trip preserves non‑code bytes.
- [ ] Hot reload tests: failing edit keeps last‑good; fixing edit swaps runner.
- [ ] Console tests: trace() lines and error render path covered.
- [x] Add README section with usage and keybinds; link test carts and debug flags.

## Keybinds (initial)
- Save: Ctrl/Cmd+S
- Run/Stop: Ctrl/Cmd+R (and optional F5)
- Find: Ctrl/Cmd+F
- Copy/Paste: Ctrl/Cmd+C/V/X
- Undo/Redo: Ctrl/Cmd+Z / Ctrl/Cmd+Shift+Z

## Testing Strategy
- Unit: code buffer ops (insert/delete/undo/redo), scroll viewport, hit‑testing.
- Integration: open→edit→save→open asserts only code changed; hot reload error gating; console logs observed.
- Manual: use existing carts (fft/vqt/time/trace) to validate livecoding flow and console output.

## Open Questions
- External file watcher for auto‑reload (deferred).
- Config: theme, font size, autosave debounce.
- Optional MRU list and single‑instance guard.
