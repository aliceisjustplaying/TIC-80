# TIC-80 Rust Rewrite — Docs Index

This folder organizes the rewrite plan, specs, architecture notes, testing strategy, and decisions. `AGENTS.md` at the repo root tracks current work and links here.

## Roadmap
- `docs/roadmap/overview.md`: High-level phased roadmap and goals (moved from RUST_REWRITE.md).
- `docs/roadmap/gui_first.md`: Combined GUI-first kickoff + milestones for `winit + pixels` and `cls/pix`.

## Specs
- `docs/specs/memory_map.md`: Canonical pointer to the root `MEMORY_MAP.md` and usage notes.
- `docs/specs/lua_api_parity.md`: API parity checklist for Lua (name, signature, side effects).
- `docs/specs/graphics.md`: Framebuffer, palette mapping, text/print semantics (stub to be expanded).
- `docs/specs/audio_fft_vqt.md`: FFT/VQT behavior and parameters (points to `CLAUDE.md`).

## Architecture
- `docs/architecture/workspace.md`: Crate layout and module boundaries.
- `docs/architecture/runtime.md`: Fixed-step loop, callbacks, and presentation responsibilities.

## Testing
- `docs/testing/strategy.md`: Testing and validation strategy across API/VRAM/audio.
- `docs/testing/frame_hashes.md`: Conventions for deterministic frame/audio hashing (stub).
- `docs/testing/test_catalog.md`: Summary of current tests and their intent.

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
