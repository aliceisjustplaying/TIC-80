# Editor Architecture (CODE/CONSOLE)

The editor renders a TIC‑80‑style UI inside the 240×136 framebuffer and handles input from `winit`. No external widget toolkit is used; this keeps rendering fully deterministic and testable.

## Rendering
- Top bar (12 px tall): tabs and buttons drawn with existing gfx primitives; button widths are computed from label length (ADV=6) and padding.
- Panels:
  - CODE: text viewport + gutter.
  - CONSOLE: placeholder; a ring buffer model will feed this view.
- Integer scaling: `winit + pixels` provides window scaling; mouse coordinates are unprojected to 240×136 for hit testing.

## Text Model
- Rope: `ropey` stores the cart source; enables fast inserts/deletes and stable indices.
- Viewport: maps `scroll_line`/`scroll_col` to rows/cols on screen; uses a 6×8 monospace grid.
  - Fixed‑width glyphs draw only the left 6 columns of the 8×8 font and advance 6 px.
  - Gutter (left 24 px) shows 1‑based line numbers.

## Caret
- TIC‑80 style: a red box slightly larger than the glyph cell with a 1px drop shadow; glyph under the caret is redrawn in dark to simulate inversion.
- Position stays aligned to the 6×8 grid; auto-scrolling keeps caret visible.

## Input
- Tabs and buttons: rectangle hit testing in framebuffer space.
- Caret navigation: arrows (more keybinds to be added).
- Editing (upcoming): insert/delete/backspace/newline; selection, undo/redo.

## Hot Reload and Cart I/O (upcoming)
- CODE‑only `.tic` save; preserve other sections; hot reload swaps `LuaRunner` and gates on errors to keep last‑good.

## Testing Strategy
- Pixel assertions for UI scaffolding and code viewport.
- Buffer operation tests for text edits and viewport mapping.
- Integration tests for `.tic` round‑trip and hot reload gating.

