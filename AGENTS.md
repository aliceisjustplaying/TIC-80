**TIC-80 Rust Rewrite — Agent Log**

- **Owner:** AI Coding Agent (Codex CLI)
- **Scope:** Help drive the Rust rewrite forward with tests-first changes, targeted features, and tight parity with TIC-80 behavior.
- **Last Updated:** 2025-08-26

**Context**
- **Rewrite code location:** All Rust rewrite code lives under `tic80_rust/` (crate root). Tests live in `tic80_rust/tests/`. The windowed demo binary is `tic80_rust/src/main.rs`.
- **Plan Docs:** `RUST_REWRITE.md` defines phased roadmap; `docs/` contains early GUI-first specs and API parity checklist.
- **Prototype Crate:** `tic80_rust` (single crate for now).
  - `gfx::framebuffer`: 240×136 palette-index framebuffer and primitives: `cls`, `pix`, `line`, `rect`, `rectb`, `circ`, `circb`, `elli`, `ellib`, `tri`, `trib`, `clip`, `print_text`, `blit_to_rgba`.
  - `core::memory`: 96 KB RAM view with `peek/poke` (1/2/4/8‑bit), `memcpy`, `memset`; VRAM screen region bridged to framebuffer.
  - `script::lua_runner`: `mlua` (vendored Lua 5.3) binding for `BOOT/TIC` and APIs: `cls/pix/line/rect/rectb/circ/circb/elli/ellib/tri/trib/clip/print` + memory helpers.
  - `main.rs`: `winit + pixels` presenter, fixed‑step tick (~60 FPS), demo cart or external `.lua`.
- **Testing:** Tests under `tic80_rust/tests/` cover framebuffer, Lua bridge, and memory; deterministic VRAM hash helpers.

**Build Hygiene (always do this)**
- Fix all compiler warnings before landing changes (treat warnings as errors).
- Run clippy and keep zero warnings: `cd tic80_rust && cargo clippy -- -D warnings`.
- Validate with tests: `cd tic80_rust && cargo test` (and run the specific failing test during fixes).
- Keep changes minimal and focused; don’t expand scope while tests are red.

**Testing**
- Strategy: see `docs/testing/strategy.md` for layers (framebuffer, Lua bridge, deterministic hashes) and future plans (audio, conformance, fuzzing).
- Catalog: see `docs/testing/test_catalog.md` for a concise list of existing tests and their intent.
- Run: `cd tic80_rust && cargo test` for unit + Lua tests; `cargo clippy -- -D warnings` for linting.
- Determinism: VRAM hashes use FNV-1a over 240×136 palette indices (see strategy doc for rationale and helper snippet).

**Decisions (Locked for prototype)**
- **Presentation:** `winit + pixels` with integer scaling (2x/3x/4x), RGBA palette conversion from 16-color default.
- **Lua Engine:** `mlua` with vendored Lua 5.3 (ADR 0003), targeting TIC-80 semantics; enable/replicate 5.1/5.2 compatibility where needed and cover with tests.
- **Framebuffer:** Single VRAM bank, palette indices in CPU memory; palette map/border/vbank deferred.

**Current Status**
- All tests pass (`cargo test`).
- Clippy clean (`cargo clippy --all-targets --all-features -D warnings`).
- Drawing primitives implemented and exposed to Lua; clipping enforced on writes; OOB reads return `nil` in Lua.
- `print` implemented with default font (variable/fixed width, scale, newline advance) and returns width.
- Memory ops (`peek/poke` 1/2/4/8‑bit, `memcpy`, `memset`) implemented; VRAM updates reflect on screen.
- CLI loads bundled default cart or a provided `.lua` path.

**Near-Term Backlog**
- Print edge cases: tests for scale>1 baseline/advance and multi‑line width parity.
- Small font: decide semantics and implement `smallfont=true` in `print` with tests.
- Lua error paths: add type/arity mismatch tests for core APIs (`pix/line/rect/print`).
- Docs: align Lua version references (5.3) and flesh out `docs/specs/graphics.md` text rules.
- Optional: add CI step for `cargo test` + clippy.

**Mid-Term Backlog**
- Sprites/tiles: `spr`, `map`, `font`, `paint`, `ttri` (with colorkey/flip/rotate/scale, remap callback).
- Banks/persistence: `vbank`, `sync`, `pmem`; palette map/border color.
- Input/system: `btn/btnp`, `key/keyp`, `mouse`, `time`, `tstamp`, `trace`, `exit`, `reset` (fixed‑step repeat timing).
- Audio: `sfx`, `music` synth/mixer; capture ring for analysis.
- Analysis: `fft/ffts/fftr/fftrs`, `vqt` variants; conformance carts + numeric tolerances.
- Platform: WASM build path; window scaling and UX polish.

**Open Questions**
- Small font (`smallfont=true`) parity: glyph source, advance, scaling, and when to land.
- `print` trimming/width: exact left/right column trimming, newline advance, and width at `scale>1`.
- Numeric types: confirm integer vs float acceptance/rounding for `line/tri` and similar APIs.
- Palette ops: `pal/palt` semantics, palette map/border color timing, and interactions with `clip`/vbank.
- Callback timing: `SCN/BDR` row timing and palette side effects integration into the pipeline.
- Determinism for `time()/tstamp()`: origin/granularity guarantees for tests.

**Operating Notes**
- Keep changes surgical and test-driven; don’t expand surface while a test is red.
- Prefer headless tests (VRAM hash or targeted pixel asserts) to validate behavior deterministically.
- Document any intentional deviations from TIC-80 behavior in this file and update `docs/api_parity_checklist.md` as needed.

**Worklog**
- 2025-08-26:
  - Reviewed `RUST_REWRITE.md` and `docs/` (API parity, GUI-first milestones).
  - Surveyed `tic80_rust` crate; confirmed implemented APIs (`cls/pix/line/rect/print`).
  - Ran tests: one failure in Lua print-origin behavior identified; queued as next fix.
  - Reorganized documentation:
    - Moved plan to `docs/roadmap/overview.md` with a root stub.
    - Merged GUI-first docs into `docs/roadmap/gui_first.md`.
    - Renamed API parity to `docs/specs/lua_api_parity.md` and added specs stubs.
    - Added docs index at `docs/README.md`, architecture/testing pages, and ADRs.
  - Added detailed testing docs (`docs/testing/strategy.md`) and a test catalog (`docs/testing/test_catalog.md`).

**Docs Index**
- Start here: `docs/README.md`
- Roadmap: `docs/roadmap/overview.md`, `docs/roadmap/gui_first.md`
- Specs: `docs/specs/memory_map.md`, `docs/specs/lua_api_parity.md`, `docs/specs/graphics.md`, `docs/specs/audio_fft_vqt.md`
- Architecture: `docs/architecture/workspace.md`, `docs/architecture/runtime.md`
- Testing: `docs/testing/strategy.md`, `docs/testing/frame_hashes.md`
- ADRs: `docs/adr/0001-winit-pixels.md`, `docs/adr/0002-mlua-lua54-compat.md` (superseded), `docs/adr/0003-lua53-with-compat.md`
