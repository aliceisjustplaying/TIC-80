# Workspace Architecture

Crates (target layout)
- `tic-core`: VM state, memory map, cart model, fixed-step ticker.
- `tic-gfx`: CPU rasterizer (`pix/line/rect/...`), VRAM/VRAM banks, palette ops.
- `tic-audio`: PSG synth + mixer; later capture buffer for FX.
- `tic-io`: Input abstraction (kbd/mouse/gamepad), FS/cart IO, time.
- `tic-api`: Language-agnostic API facade matching TIC-80 surface.
- `tic-lua`: `mlua` embedding and Lua shims over `tic-api`.
- `tic-fx`: FFT/VQT analysis per CLAUDE.md.
- `tic-winit` (or `tic-sdl`): Platform layer for window/audio/input.
- `tic-runner`: CLI binary to run carts headless or with window.

Data Flow
- Lua (`tic-lua`) calls into `tic-api` → forwards to `tic-core/gfx/audio/io`.
- `tic-gfx` writes to VRAM page(s); presenter converts palette indices to RGBA for display.
- `tic-audio` produces sample blocks; optional capture ring shared with `tic-fx`.

