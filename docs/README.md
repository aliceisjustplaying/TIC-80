# TIC-80 Rust Rewrite — Docs Index

This folder organizes the rewrite plan, specs, architecture notes, testing strategy, and decisions. `AGENTS.md` at the repo root tracks current work and links here.

## Roadmap
- `docs/roadmap/overview.md`: High-level phased roadmap and goals (moved from RUST_REWRITE.md).
- `docs/roadmap/gui_first.md`: Combined GUI-first kickoff + milestones for `winit + pixels` and `cls/pix`.
- `docs/roadmap/editor_livecoding.md`: Livecoding editor plan (TIC‑80 UI vibes): CODE + CONSOLE only.
 - `docs/roadmap/todos_code_review.md`: Rolling TODOs from code review (high/medium/low priority) with checkboxes.

## Specs
- `docs/specs/memory_map.md`: Canonical pointer to the root `MEMORY_MAP.md` and usage notes.
- `docs/specs/lua_api_parity.md`: API parity checklist for Lua (name, signature, side effects).
- `docs/specs/graphics.md`: Framebuffer, palette mapping, text/print semantics (stub to be expanded).
- `docs/specs/audio_fft_vqt.md`: FFT/VQT behavior and Rust implementation plan (cpal + realfft), with links to `CLAUDE.md`.
- `docs/specs/implementation_status.md`: What’s implemented vs pending, with notes on behavior.

## Architecture
- `docs/architecture/workspace.md`: Crate layout and module boundaries.
- `docs/architecture/runtime.md`: Fixed-step loop, callbacks, and presentation responsibilities.
- `docs/architecture/clippy_policy.md`: Lint policy (pedantic baseline + curated allows).

## Testing
- `docs/testing/strategy.md`: Testing and validation strategy across API/VRAM/audio.
- `docs/testing/frame_hashes.md`: Conventions for deterministic frame/audio hashing (stub).
- `docs/testing/test_catalog.md`: Summary of current tests and their intent.
- `docs/testing/test_carts.md`: Manual test carts (Lua) for quick verification (FFT, time/trace).

## Decisions (ADR)
- `docs/adr/0001-winit-pixels.md`: Windowing/presentation stack decision.
- `docs/adr/0002-mlua-lua54-compat.md`: Lua 5.4 choice (superseded).
- `docs/adr/0003-lua53-with-compat.md`: Lua 5.3 with 5.1/5.2 compatibility.

Notes
- `MEMORY_MAP.md` at repo root remains the canonical reference for layout. Specs here link to it rather than duplicating.
- `CLAUDE.md` remains at repo root; the spec page links to it for detailed FFT/VQT behavior.

## Run the Demo
- Default cart:
  - From crate dir: `cd tic80_rust && cargo run`
  - From repo root: `cargo run --manifest-path tic80_rust/Cargo.toml`
- Load a `.lua` file:
  - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/alt.lua`
  - In crate dir: `cargo run -- assets/alt.lua`
- Audio FFT test cart:
  - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/fft_test.lua --audio-device "<name-substr>"`
  - Add `--audio-vu` to print a 1s peak; add `--debug-fft` to print the first few bins.
- Time/Trace test cart:
  - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/time_trace_test.lua`
  - Shows elapsed ms and emits a trace once per second to the console.
- VQT test cart:
  - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/vqt_test.lua --audio-device "<name-substr>"`
  - Visualizes 120 bins (12 octaves) with a 2px per-bin bar chart; auto-toggles between raw and whitened views every ~3 seconds.

## CLI
- Usage: `tic80_rust [OPTIONS] [CART.lua]`
- Options:
  - `-h, --help`: Show help and exit.
  - `--quiet`: Suppress once-only warnings and Lua BOOT()/TIC() error prints (useful for headless runs).
  - `--list-audio`: List input audio devices and exit.
  - `--audio-device <SUBSTR>`: Select input device by substring match (case-insensitive).
  - `--audio-disable`: Disable audio capture and analysis.
  - `--audio-vu`: Print VU peak dBFS once per second.
  - `--debug-fft`: Print the first 16 FFT bins (smoothed, normalized) roughly every 500 ms.
  - `--debug-fx`: Print per-second FX timings and ring stats: pushed delta (dp), overflow delta/total (ovf), underrun delta/total, consumed delta (cons), EMA samples/tick, and estimated ring occupancy.
- Notes:
  - CART.lua is optional; defaults to the bundled cart if omitted.
  - Audio capture picks the nearest supported sample rate to 44100 Hz and logs the selection.
  - Window is fixed 240×136 internal res with integer scaling.

## Debug Flags (Audio Analysis)
- `--debug-fft`: First 16 FFT bins (smoothed normalized).
- `--debug-fx`: FX timings plus ring stats (dp/ovf/underrun/cons), EMA samples/tick, estimated occupancy.
