# Test Catalog

This document summarizes the current test coverage with file paths and intent.

## Framebuffer Unit Tests
- `tic80_rust/tests/gfx_framebuffer_tests.rs`
  - `cls_fills_entire_buffer`: `cls` fills the entire VRAM with the color index.
  - `pix_read_write_and_bounds`: `pix` read/write semantics; OOB reads return `None` and writes are ignored.
  - `rect_fill_and_clipping`: `rect` fills and clips to viewport; fully OOB rects are no‑ops.
  - `line_basic_counts_and_endpoints`: Line endpoints colored; counts match `max(dx,dy)+1` both directions.
  - `blit_to_rgba_maps_palette`: Palette index→RGBA mapping matches Sweetie16 sRGB bytes (white=idx12, greys idx13/14/15).
  - `rectb_draws_border`: `rectb` draws a 1‑px border; interior remains unchanged.
  - `clip_limits_drawing_and_reset`: Clip restricts drawing; reset restores full viewport.
  - `print_width_fixed_vs_variable_and_newline`: `print_text` width (fixed vs variable), newline row advance, and scale behavior.
  - `clip_affects_pix_write`: `pix(x,y,color)` respects the active clip.
  - `robust_oob_line_and_rectb`: OOB line still draws in-bounds; border rect crossing viewport produces in-bounds edges.
  - `circb_cardinals_and_oob`: Circle border’s cardinal points lit; just outside remains background.
  - `circ_fill_center_row_and_clip`: Filled center row span and clip restriction.
  - `circ_zero_radius_draws_center`: r=0 draws a point for circ/circb.
  - `ellib_cardinals_and_fill_center_row`: Ellipse border cardinals; filled center row interior.
  - `tri_fill_and_border`: Basic filled triangle and border overlay.
  - `tri_top_left_flat_top_inclusion`: Top-left rule on flat-top triangles (endpoints excluded on top edge).
  - `tri_top_left_flat_bottom_exclusion`: Bottom edge excluded on flat-bottom triangles.
  - `tri_adjacent_rect_no_gaps`: Two triangles tile a rectangle without gaps.
  - `tri_shared_edge_shallow_slope_tiles_rect`: Shallow-slope shared-edge tiling covers rectangle area exactly.
  - `tri_degenerate_zero_area_draws_nothing`: Collinear triangles draw nothing.
  - `ellib_cardinals_and_fill_center_row`: Ellipse border cardinals; filled center row interior.
  - `ellib_cardinal_points_aspect_wide`: Wide-aspect ellipse draws expected cardinals; neighbors remain background.

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
  - `lua_circ_and_circb`: Circle fill and border via Lua.
  - `lua_elli_ellib_and_tri_trib`: Ellipse and triangle APIs via Lua.
  - `lua_line_float_truncates_to_integer_pixels`: Float coordinates truncate to integer pixels identically to TIC-80.

## Memory Tests
- `tic80_rust/tests/memory_tests.rs`
  - `poke4_sets_framebuffer_pixel`: 4-bit writes update screen pixels.
  - `peek4_reads_back_nibble`: 4-bit reads reflect framebuffer.
  - `memcpy_and_memset_affect_vram`: VRAM writes via memcpy/memset reach the screen.
  - `peek_poke_bits_general_ram`: 1/4-bit addressing in general RAM behaves correctly.
  - `vram_writes_ignore_clip`: VRAM screen mapping writes ignore clip and update pixels directly.
  - `two_bit_cross_byte_alignment`: 2-bit writes at end of a byte and start of next do not bleed.
  - `four_bit_unaligned_nibbles`: 4-bit nibble writes across/within bytes pack correctly.
- `tic80_rust/tests/memory_bits_roundtrip.rs`
  - `roundtrip_peek_poke_bits_general_ram`: Round-trip property-like checks for 1/2/4/8-bit peek/poke across a RAM window.
  - `vram_screen_boundary_write_does_not_bleed`: Last screen byte maps to the last two pixels; next byte (non-screen VRAM) does not affect framebuffer.
- `tic80_rust/tests/memory_bits_alignment.rs`
  - `two_bit_cross_byte_alignment`: 2‑bit writes at end of one byte and start of next do not bleed.
  - `four_bit_unaligned_nibbles`: Odd/even nibble writes across bytes and within a byte pack correctly.

## FFT Tests
- `tic80_rust/tests/fft_tests.rs`
  - `fft_query_peak_at_bin`: Bin-aligned sine produces a distinct raw peak at the expected bin versus neighbors.
  - `lua_fft_returns_normalized_bin`: Verifies Lua `fft(k)` returns normalized magnitude by gating a pixel.
  - `fft_query_range_clamps_and_sums`: Clamping and inclusive sum behavior matches C (OOB handling and range sums).
  - `fft_single_bin_peak_and_normalization`: Single-bin sine produces clear raw peak and normalized near-1.0 at bin.
  - `vqt_bin_has_higher_energy_than_neighbors`: Sine at a center frequency yields higher raw energy at the target bin than neighbors.

Notes
- Tests prefer headless framebuffer inspection over image baselines.
- Hashing uses FNV‑1a over VRAM palette indices for portability and stability.
- For manual carts, see `docs/testing/test_carts.md`.

## Editor Tests
- `tic80_rust/tests/editor_smoke.rs`: UI shell draws and tabs switch on click; verifies top-bar pixels.
- `tic80_rust/tests/editor_code_view_tests.rs`: CODE viewport renders gutter digits and text cells.
- `tic80_rust/tests/editor_editing_tests.rs`: basic editing behavior for insert/newline, backspace (join-prev), delete (join-next), Home/End, and Tab-as-spaces.
- `tic80_rust/tests/editor_selection_undo_tests.rs`: selection replace/cut/paste and undo/redo cycles; select-all.
- `tic80_rust/tests/editor_selection_shadow_tests.rs`: multi-line selection renders without interior bottom seams; right-edge shadow is exactly 7 px tall.
- `tic80_rust/tests/editor_selection_align_tests.rs`: selection top aligns with caret box (small font baseline at 6 px, pitch 7 px).
- `tic80_rust/tests/editor_page_nav_tests.rs`: PageUp/Down movement by visible lines; Shift variants extend selection.
- `tic80_rust/tests/editor_ctrl_home_end_tests.rs`: Ctrl/Cmd+Home/End jump to document bounds.
- `tic80_rust/tests/editor_indent_outdent_tests.rs`: Block indent/outdent adds/removes a leading space per selected line.

## Screenshot Tests
- `tic80_rust/tests/screenshot_smoke.rs`
  - `save_scaled_png_has_expected_dimensions`: Draw to framebuffer, encode/decode PNG in-memory; save a 3x scaled PNG to temp and verify dimensions.
- `tic80_rust/tests/e2e_headless_cli.rs`
  - `e2e_headless_default_cart_screenshot`: Runs the CLI binary with `--headless --screenshot` and decodes the PNG.
  - `e2e_headless_editor_screenshot`: Runs with `--headless --editor --screenshot` and verifies the PNG.
- `tic80_rust/tests/editor_word_nav_tests.rs`: Ctrl/Alt word navigation and word deletion (left/right).
- `tic80_rust/tests/editor_smart_home_doc_select_tests.rs`: Smart Home toggle and Ctrl/Cmd+Shift+Home/End document selection.
