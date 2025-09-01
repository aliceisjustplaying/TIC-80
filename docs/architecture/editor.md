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

## Selection & Shadows
- Selection fill uses palette index 14 (grey, Sweetie16) and respects the 6×8 cell grid.
- Drop shadow (palette 0) is rendered only on the outer perimeter:
  - Right edge: draw a 1px vertical line unless the next row’s selection extends strictly further right (to keep the outer edge clean).
  - Bottom edge: drawn per-segment by subtracting the next row’s covered columns. If the next row overlaps, no interior horizontal seam is drawn; only left/right overhangs render a bottom shadow.
  - This matches TIC‑80’s visual continuity for multi-line selections (no interior seams).

## Top Bar & Layout
- Toolbar height: 7 px (1 px margins around 6 px small font).
- Title: left‑aligned “CODE” in grey (palette 14); no shadow in the current build (can be re‑enabled later).
- Background: white (palette 12).
- Gutter: 3 digits (18 px) width; a 1 px gap separates gutter and code.
- Text grid: 6 px small font on a 7 px line pitch; first code row starts immediately under the toolbar (no extra padding).

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
