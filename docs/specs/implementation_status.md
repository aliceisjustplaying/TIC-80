# Implementation Status (Rust Rewrite)

This page tracks which TIC-80 APIs are implemented in the Rust rewrite, notes per function, and what remains.

Implemented (Lua + Core)
- Drawing
  - `cls(color)`: Full-screen clear (no clip). Note: masks color to 0..15.
  - `pix(x,y[,color])`: Read returns color or `nil` when OOB; write obeys clip; reads ignore clip.
  - `line(x0,y0,x1,y1,color)`: Bresenham with inclusive endpoints; clip respected.
  - `rect(x,y,w,h,color)`: Filled; respects viewport and clip.
  - `rectb(x,y,w,h,color)`: One-pixel border; inclusive; clip respected.
  - `circ(cx,cy,r,color)`: Filled; r=0 is a point; clip respected.
  - `circb(cx,cy,r,color)`: Border; r=0 is a point; clip respected.
  - `elli(cx,cy,a,b,color)`: Filled; handles degenerate axes; clip respected.
  - `ellib(cx,cy,a,b,color)`: Border; clip respected.
  - `tri(x1,y1,x2,y2,x3,y3,color)`: Filled triangle with top-left rule; no gaps with adjacent triangles.
  - `trib(x1,y1,x2,y2,x3,y3,color)`: Border triangle via lines.
  - `print(text,x,y,color=15,fixed=false,scale=1,small=false) -> width`: Default font, variable-width trim, scale applied to return width; newlines advance by 6 px (times scale).
- Clip
  - `clip(x,y,w,h)` and `clip()`: Set/reset clip rectangle affecting all draw writes; reads are unaffected.
 - Memory
  - `peek(addr[,bits=8])`, `poke(addr, value[,bits=8])`: 8/4/2/1-bit addressing across full 96 KB; VRAM screen region mapped to live framebuffer (nibble-packed 2 px/byte).
  - `peek1/peek2/peek4`, `poke1/poke2/poke4`: Bit-specific helpers.
  - `memcpy(dst, src, size)`, `memset(dst, value, size)`: Byte-wise operations; overlap-safe memcpy; VRAM ops update on-screen pixels immediately.

Implemented (Runner/CLI)
- `.lua` loader: First CLI arg as a `.lua` path runs external script; fallback to bundled `assets/default.lua`.
- Window title: “rustic”.
- Audio capture scaffolding: `cpal` input stream (44.1 kHz if supported), stereo→mono downmix, lock‑free ring buffer (8192 samples); CLI flags to list/select devices and optional VU meter output.
 - VU behavior: Peak meter observed ~-180 dBFS at silence (BlackHole 2ch on macOS), responsive under Multi‑Output device routing.

Implemented (Analysis)
- FFT (2k):
  - Real‑to‑complex transform using `realfft` over the latest 2048 samples on the tick thread.
  - Bins 0..1023 maintained (Nyquist dropped), magnitudes scaled by 2.0 to match C behavior.
  - Buffers: raw, raw‑smoothed (0.6), normalized, normalized‑smoothed, with peak tracking (`fPeakMin=0.01`, `fPeakSmooth=0.995`).
  - Optional `--debug-fft` prints the first 16 smoothed normalized bins periodically.
  - Lua APIs implemented: `fft/ffts/fftr/fftrs` with C-identical clamping/sum semantics.
  - Tests: headless unit tests cover raw-peak behavior and Lua bridging; additional clamp/sum range tests added.

Behavioral Notes
- Triangles: top-left inclusion; CCW orientation enforced internally; half-open bounding box prevents shared-edge double draws.
- Ellipses vs circles: fill then border may overdraw endpoints; order-dependent at axis rows (parity with TIC-80).
- Font: Default TIC-80 bitmap included; LSB-left bits; 6 px advance; trimming for variable width.

Pending APIs (not implemented yet)
- Texturing/tiles
  - `spr`, `map`, `ttri`, `font` (RAM font region), `paint`.
  - Chroma/colorkey handling, flips/rotate/scale for sprites, map remap callbacks.
- Memory/banks
  - `vbank`, `sync`, `pmem`.
- Input/system
  - `btn/btnp`, `key/keyp`, `mouse`, `time`, `tstamp`, `trace`, `exit`, `reset`.
- Audio
  - `sfx`, `music`; audio mixer/synth; capture ring for analysis.
- Analysis
  - `vqt/vqts/vqtr/vqtrs` and whitening variants; behavior per CLAUDE.md.
- Sprite flags
  - `fget`, `fset`.

Test Coverage (summary)
- Framebuffer unit tests cover: cls/pix/line/rect/rectb/circ/circb/elli/ellib/tri/trib, clip behavior, OOB, print width/newlines, palette blit, triangle edge rules.
- Lua bridge tests cover: cls/pix/line/rect/print/rectb/clip/circ/circb/elli/ellib/tri/trib and cart file loading.
- Determinism: simple VRAM hash helper asserts stable hashes over N ticks for the default cart.

See also
- Graphics semantics: `docs/specs/graphics.md`.
- API parity checklist: `docs/specs/lua_api_parity.md`.
- Testing strategy and catalog: `docs/testing/strategy.md`, `docs/testing/test_catalog.md`.
