# TIC-80 Lua API Parity Checklist

Authoritative references: MEMORY_MAP.md (root), src/api.h (TIC_API_LIST), src/api/luaapi.c.
Goal: ensure Rust implementation preserves names, signatures, return types, side effects, and timing.

For each API, we list the expected signature (per spec), key side effects, and the Rust subsystem owner.

## Callbacks

- TIC: `TIC()`
  - Effect: Per-frame tick at 60 FPS.
  - Subsystem: tic-core (scheduler), tic-lua (callback wiring).
- BOOT: `BOOT`
  - Effect: One-time init after loading cart before first TIC.
  - Subsystem: tic-core, tic-lua.
- SCN: `SCN(row)`
  - Effect: Scanline callback during draw; palette tricks.
  - Subsystem: tic-gfx (scanline timing), tic-lua.
- BDR: `BDR(row)`
  - Effect: Border scanline callback; palette tricks.
  - Subsystem: tic-gfx, tic-lua.
- MENU: `MENU(index)`
  - Effect: Game menu handler.
  - Subsystem: tic-core (menu routing), tic-lua.

## Drawing

- print: `print(text x=0 y=0 color=15 fixed=false scale=1 smallfont=false) -> width`
  - Effect: Draw text; returns width.
  - Subsystem: tic-gfx.
- cls: `cls(color=0)`
  - Effect: Clear screen to color.
  - Subsystem: tic-gfx.
- pix: `pix(x y color)` / `pix(x y) -> color`
  - Effect: Read or write pixel.
  - Subsystem: tic-gfx (VRAM).
- line: `line(x0 y0 x1 y1 color)`
  - Effect: Draw line.
  - Subsystem: tic-gfx.
- rect: `rect(x y w h color)`
  - Effect: Filled rectangle.
  - Subsystem: tic-gfx.
- rectb: `rectb(x y w h color)`
  - Effect: Rectangle border.
  - Subsystem: tic-gfx.
- circ: `circ(x y radius color)`
  - Effect: Filled circle.
  - Subsystem: tic-gfx.
- circb: `circb(x y radius color)`
  - Effect: Circle border.
  - Subsystem: tic-gfx.
- elli: `elli(x y a b color)`
  - Effect: Filled ellipse.
  - Subsystem: tic-gfx.
- ellib: `ellib(x y a b color)`
  - Effect: Ellipse border.
  - Subsystem: tic-gfx.
- tri: `tri(x1 y1 x2 y2 x3 y3 color)`
  - Effect: Filled triangle.
  - Subsystem: tic-gfx.
- trib: `trib(x1 y1 x2 y2 x3 y3 color)`
  - Effect: Triangle border.
  - Subsystem: tic-gfx.
- ttri: `ttri(x1 y1 x2 y2 x3 y3 u1 v1 u2 v2 u3 v3 use_map=false chroma=nil z1=0 z2=0 z3=0)`
  - Effect: Textured triangle (tiles or explicit texture), optional chroma table, optional map source.
  - Subsystem: tic-gfx (texturing), tic-core (remap helper).
- paint: `paint(x y color [bordercolor])`
  - Effect: Flood fill (with optional border color).
  - Subsystem: tic-gfx.
- clip: `clip(x y w h)` and `clip()`
  - Effect: Set/reset clipping rectangle.
  - Subsystem: tic-gfx.
- spr: `spr(id x y colorkey=-1 scale=1 flip=0 rotate=0 w=1 h=1)`
  - Effect: Draw sprite or composite sprite region; supports flip/rotate/scale; optional colorkey.
  - Subsystem: tic-gfx (sprite blitter), respects palette map and vbank.
- map: `map(x=0 y=0 w=30 h=17 sx=0 sy=0 colorkey=-1 scale=1 remap=nil)`
  - Effect: Draw map region to screen with optional remap callback.
  - Subsystem: tic-gfx; remap callback via tic-lua.
- font: `font(text x y chromakey char_w char_h fixed=false scale=1 alt=false) -> width`
  - Effect: Draw text using font region in RAM; returns width.
  - Subsystem: tic-gfx.

## Tilemap Access

- mget: `mget(x y) -> tile_id`
  - Effect: Read tile id at map cell (x,y).
  - Subsystem: tic-gfx (map RAM view), tic-core.
- mset: `mset(x y tile_id)`
  - Effect: Write tile id at map cell (x,y); persistent only after `sync()`.
  - Subsystem: tic-gfx (map RAM view), tic-core.

## Input

- btn: `btn(id) -> pressed`
  - Effect: Gamepad button state (held).
  - Subsystem: tic-io (gamepad).
- btnp: `btnp(id hold=-1 period=-1) -> pressed`
  - Effect: Gamepad pressed/auto-repeat.
  - Subsystem: tic-io (edge detection + repeat timing).
- key: `key(code=-1) -> pressed`
  - Effect: Keyboard key state (held); `-1` checks any.
  - Subsystem: tic-io (keyboard).
- keyp: `keyp(code=-1 hold=-1 period=-1) -> pressed`
  - Effect: Keyboard pressed/auto-repeat.
  - Subsystem: tic-io.
- mouse: `mouse() -> x y left middle right scrollx scrolly`
  - Effect: Mouse state.
  - Subsystem: tic-io (mouse).

## Memory and Banks

- peek: `peek(addr bits=8) -> value`
- poke: `poke(addr value bits=8)`
- peek1/peek2/peek4: `peek1(addr)`, `peek2(addr)`, `peek4(addr)`
- poke1/poke2/poke4: `poke1(addr value)`, `poke2(addr value)`, `poke4(addr value)`
  - Effect: Read/write RAM/VRAM according to MEMORY_MAP.md; 1/2/4-bit and nibble addressing semantics preserved.
  - Subsystem: tic-core (memory), tic-gfx (VRAM effects).
- memcpy: `memcpy(dest source size)`
- memset: `memset(dest value size)`
  - Effect: Raw memory block ops across 96KB RAM.
  - Subsystem: tic-core.
- vbank: `vbank(bank) -> prev` or `vbank() -> prev`
  - Effect: Switch active 16KB VRAM bank (0 or 1); returns previous bank.
  - Subsystem: tic-gfx (VRAM pages), tic-core (state).
- sync: `sync(mask=0 bank=0 tocart=false)`
  - Effect: Copy between cart banks and runtime; respects mask (tiles, sprites, map, sfx, music, palette, flags, screen).
  - Subsystem: tic-core (cart/runtime memory), tic-io (persistence).
- pmem: `pmem(index value)` / `pmem(index) -> value`
  - Effect: Read/write 256×u32 persistent slots; cart-hash keyed.
  - Subsystem: tic-core (persistent store).

## Text/Console and System

- trace: `trace(message color=15)`
  - Effect: Print to console (not screen) in color.
  - Status: Implemented (color accepted; prints to console; tests use internal buffer).
  - Subsystem: tic-core (logger).
- time: `time() -> ticks`
  - Effect: Milliseconds since cart start (double); used for animation/timing.
  - Status: Implemented (monotonic; tick-thread time origin).
  - Subsystem: tic-core (timer).
- tstamp: `tstamp() -> timestamp`
  - Effect: Seconds since Unix epoch.
  - Subsystem: tic-core (system clock proxy).
- exit: `exit()`
  - Effect: Return to console when TIC ends.
  - Subsystem: tic-core (control flow).
- reset: `reset()`
  - Effect: Reset cart runtime (not process exit).
  - Subsystem: tic-core.

## Audio

- sfx: `sfx(id note=-1 duration=-1 channel=0 volume=15 speed=0)`
  - Effect: Play/stop SFX on channel; supports id -1 to stop; note as int or "C#-4" style.
  - Subsystem: tic-audio (synth/mixer).
- music: `music(track=-1 frame=-1 row=-1 loop=true sustain=false tempo=-1 speed=-1)`
  - Effect: Start/stop music playback; -1 to stop; supports overrides.
  - Subsystem: tic-audio (tracker).

## Analysis (FFT / VQT)

- fft: `fft(start_freq end_freq=-1)`
- ffts: `ffts(start_freq end_freq=-1)`
- fftr: `fftr(start_freq end_freq=-1)`
- fftrs: `fftrs(start_freq end_freq=-1)`
  - Effect: FFT magnitude queries (peak-normalized vs raw; smoothed vs raw) per CLAUDE.md; 1024 bins, 2,048-sample window ~21 FPS.
  - Subsystem: tic-fx (FFT), tic-audio (capture buffer).
- vqt: `vqt(bin)` / vqts/vqtr/vqtrs/vqtw/vqtsw/vqtrw/vqtrsw
  - Effect: VQT magnitude queries (normalized/raw; smoothed/raw; whitened/non); 120 bins, 8,192-sample window ~5.4 FPS.
  - Subsystem: tic-fx (VQT), tic-audio (capture buffer).

## Sprite Flags

- fget: `fget(sprite_id flag) -> bool`
- fset: `fset(sprite_id flag bool)`
  - Effect: Per-sprite flag bits 0..7.
  - Subsystem: tic-gfx (sprite metadata in RAM), tic-core.

## Input Mapping and Menu

- MENU(index): see Callbacks
  - Effect: Handle game menu actions.
  - Subsystem: tic-core (menu controller).

---

Verification plan per API:
- Signature/arity: Match src/api.h docs and Lua binding arity checks.
- Return types: Match numeric vs boolean vs string; e.g., time() returns double.
- Side effects: Test VRAM/memory deltas, audio start/stop, persistent pmem behavior.
- Timing: Respect fixed step; SCN/BDR callbacks invoked with correct rows.
- Memory map: All peek/poke/mem ops constrained to MEMORY_MAP.md ranges; vbank switching affects VRAM offsets.

Notes:
- Deprecated: `textri` exists under BUILD_DEPRECATED; keep optional compat layer if needed.
- Ensure `LUA_COMPAT_5_2`-equivalent behavior to match current build semantics for Lua.
