# Test Catalog

This document summarizes the current test coverage with file paths and intent.

## Framebuffer Unit Tests
- `tic80_rust/tests/gfx_framebuffer_tests.rs`
  - `cls_fills_entire_buffer`: `cls` fills the entire VRAM with the color index.
  - `pix_read_write_and_bounds`: `pix` read/write semantics; OOB reads return `None` and writes are ignored.
  - `rect_fill_and_clipping`: `rect` fills and clips to viewport; fully OOB rects are no‑ops.
  - `line_basic_counts_and_endpoints`: Line endpoints colored; counts match `max(dx,dy)+1` both directions.
  - `blit_to_rgba_maps_palette`: Palette index→RGBA mapping matches expected sRGB bytes.
  - `rectb_draws_border`: `rectb` draws a 1‑px border; interior remains unchanged.
  - `clip_limits_drawing_and_reset`: Clip restricts drawing; reset restores full viewport.
  - `print_width_fixed_vs_variable_and_newline`: `print_text` width (fixed vs variable), newline row advance, and scale behavior.
  - `clip_affects_pix_write`: `pix(x,y,color)` respects the active clip.
  - `robust_oob_line_and_rectb`: OOB line still draws in-bounds; border rect crossing viewport produces in-bounds edges.

## Lua Bridge Tests
- `tic80_rust/tests/lua_api_tests.rs`
  - `lua_cls_and_pix`: `BOOT` clears; `TIC` writes a pixel; verify both states.
  - `lua_line_and_rect`: Lines and filled rect via Lua.
  - `lua_print_width_marker`: `print` returns width; script uses it to place a marker.
  - `lua_print_defaults_and_pix_read`: `print` defaults and `pix` read mode; verifies glyphs drawn near origin.
  - `lua_clip_and_rectb`: `clip()` and `rectb` via Lua; clip restricts drawing.
  - `lua_runs_alt_cart_file`: Loads `assets/alt.lua`, checks background/marker/square.
  - `lua_pix_oob_read_returns_nil`: OOB `pix` read returns `nil` in Lua.
  - `lua_default_cart_deterministic_hash`: Default cart produces deterministic frame hashes for fixed tick counts.

Notes
- Tests prefer headless framebuffer inspection over image baselines.
- Hashing uses FNV‑1a over VRAM palette indices for portability and stability.

