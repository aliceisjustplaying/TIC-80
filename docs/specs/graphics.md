# Graphics Spec

Scope
- Framebuffer: 240×136, 8-bit palette indices (0..15) per pixel.
- Palette: 16 sRGB entries; index→RGBA conversion for presentation; no color-space transforms.
- Text: Default font; 6 px advance within an 8×8 glyph box; variable-width by trimming empty columns when `fixed=false`.
- Clip: Active clip rectangle constrains all drawing writes; reads are unaffected.
 - VRAM mapping: Screen region writes via memory (`poke`/`memcpy`/`memset`) are not affected by `clip` and update pixels directly.

Implemented Semantics
- `cls(color=0)`: Fills framebuffer with palette index (masked to 0..15). Honors clip by design via `set_pixel` usage in higher-level draws; `cls` itself fills full screen (like TIC-80).
- `pix(x,y[,color])`:
  - Read mode: returns palette index at `(x,y)` or `nil` (Lua)/`None` (Rust) when OOB. Reads ignore `clip`.
  - Write mode: writes masked color if `(x,y)` is in-bounds and inside the current `clip`; otherwise ignored.
- `line(x0,y0,x1,y1,color)`: Integer Bresenham; inclusive endpoints; obeys `clip` through `set_pixel`.
- `rect(x,y,w,h,color)`: Filled rect; clips to viewport and active `clip`.
- `rectb(x,y,w,h,color)`: One-pixel border; inclusive edges; clipped by `clip`.
- `circ(cx,cy,r,color)`: Filled circle via symmetric horizontal spans; r=0 draws a point; obeys `clip`.
- `circb(cx,cy,r,color)`: One-pixel border using midpoint algorithm; r=0 draws a point; obeys `clip`.
- `elli(cx,cy,a,b,color)`: Filled ellipse via horizontal spans; handles degenerate `a=0` (vertical line), `b=0` (horizontal line), and `a=b=0` (point); obeys `clip`.
- `ellib(cx,cy,a,b,color)`: One-pixel border via midpoint; handles `a=b=0` as a point; obeys `clip`.
- `tri(x1,y1,x2,y2,x3,y3,color)`: Filled triangle using edge functions at pixel centers with a top-left fill rule:
  - Include pixels on top and left edges; exclude pixels on bottom and right edges.
  - Processes CCW orientation internally for consistent edge testing; no gaps when tiling adjacent triangles.
  - Obeys `clip` through `set_pixel`.
- `trib(x1,y1,x2,y2,x3,y3,color)`: One-pixel border using three inclusive Bresenham lines; edges meet at vertices; obeys `clip`.
- `print(text, x=0, y=0, color=15, fixed=false, scale=1, small=false) -> width`:
  - Font: default TIC-80 bitmap; bit order LSB-left; 8×8 glyph box; 6 px advance; variable-width trimming when `fixed=false`.
  - Origin: top-left of first drawn column is `(x,y)`.
  - Scaling: draws scaled glyphs; returned width includes scaling.
  - Newlines: advances by 6 px per line (scale applied); `smallfont` currently unused.
  - Width: returns the width of the longest line (after trimming when `fixed=false`). A one-pixel spacing is applied between variable-width glyphs.

Clip Behavior
- `clip(x,y,w,h)`: Sets active clip rectangle; `clip()` resets to full screen.
- All draw functions respect `clip` via `set_pixel`/`hspan`; `pix` reads ignore `clip`.

Notes
- Triangle parity: top-left rule ensures bit-for-bit agreement with TIC-80 edge inclusion and eliminates gaps with adjacent primitives.
- Ellipse parity: on rows coinciding with axes (e.g., center row), `ellib` perimeter overlaps `elli` endpoints; resulting color depends on draw order (expected in TIC-80 as well).
- `blit_to_rgba`: Converts framebuffer indices to RGBA using the 16-color palette (no gamma adjustments).

Pending (not implemented yet)
- `spr`, `map`, `ttri`, `paint`, `font` API variant, palette map, border color, `vbank`.
- Blending rules, chroma tables, and remap callbacks for textured triangles and map drawing.
