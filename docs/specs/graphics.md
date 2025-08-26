# Graphics Spec (Stub)

Scope
- Framebuffer: 240×136, 8-bit palette indices (0..15) per pixel.
- Palette: 16 sRGB entries; index→RGBA conversion for presentation; no color-space transforms.
- Text: Default font (5×8 advance within 8×8 glyph box), variable-width by trimming empty columns when `fixed=false`.

Semantics (to expand)
- `cls(color=0)`: Fill the framebuffer with palette index (masked to 0..15).
- `pix(x,y[,color])`: Read returns current index or nil when OOB; write masks to 0..15 and ignores OOB.
- `line/rect/rectb`: Integer rasterization; inclusive endpoints for lines; clipping to framebuffer bounds.
- `print(text, x=0, y=0, color=15, fixed=false, scale=1, small=false) -> width`:
  - Uses default font bitmap; top-left of first drawn column is `(x,y)`.
  - Returns drawn width in pixels (pre-scale), consistent with TIC-80.

Open items
- Document exact TIC-80 font bit packing and baseline to ensure parity.
- Add rules for `clip`, `palette map`, `vbank`, `border`, and blitters.

