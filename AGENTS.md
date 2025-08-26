**TIC-80 Rust Rewrite — Agent Log**

- **Owner:** AI Coding Agent (Codex CLI)
- **Scope:** Help drive the Rust rewrite forward with tests-first changes, targeted features, and tight parity with TIC-80 behavior.
- **Last Updated:** 2025-08-26

**Context**
- **Rewrite code location:** All Rust rewrite code lives under `tic80_rust/` (crate root). Tests live in `tic80_rust/tests/`. The windowed demo binary is `tic80_rust/src/main.rs`.
- **Plan Docs:** `RUST_REWRITE.md` defines phased roadmap; `docs/` contains early GUI-first specs and API parity checklist.
- **Prototype Crate:** `tic80_rust` (single crate for now).
  - `gfx::framebuffer`: 240×136 u8-index framebuffer; `cls`, `pix`, `line` (Bresenham), `rect`, `blit_to_rgba`, `print_text` (default font).
  - `script::lua_runner`: `mlua` (Lua 5.4, vendored) binding for `cls/pix/line/rect/print` + `BOOT/TIC` flow.
  - `main.rs`: `winit + pixels` presenter, fixed-step tick (~60 FPS), demo cart.
- **Testing:** Rust unit tests under `tic80_rust/tests/` covering gfx and minimal Lua APIs.

**Build Hygiene (always do this)**
- Fix all compiler warnings before landing changes (treat warnings as errors).
- Run clippy on the crate and keep zero warnings: `cd tic80_rust && cargo clippy --all-targets --all-features -D warnings`.
- Validate with tests: `cd tic80_rust && cargo test` (and run the specific failing test during fixes).
- Keep changes minimal and focused; don’t expand scope while tests are red.

**Decisions (Locked for prototype)**
- **Presentation:** `winit + pixels` with integer scaling (2x/3x/4x), RGBA palette conversion from 16-color default.
- **Lua Engine:** `mlua` with vendored Lua 5.4, behavior aligned to 5.2 where needed (compat noted in docs).
- **Framebuffer:** Single VRAM bank, palette indices in CPU memory; palette map/border/vbank deferred.

**Current Status**
- Minimal end-to-end loop works: window opens, demo script runs (`cls/pix/line/rect/print`).
- Tests run locally; 1 Lua API test currently failing (see below).
- Docs/specs in place for early milestones: GUI + `cls/pix` and Lua + `cls/pix`.

**Failing Test (Next Task)**
- `tic80_rust/tests/lua_api_tests.rs::lua_print_defaults_and_pix_read`
  - Symptom: After `print("A")` at (0,0), script checks `pix(0,0)==15` to place a marker at `(w,0)`. Marker not found on row 0.
  - Likely Cause: Glyph rendering/metrics mismatch at origin. Candidates:
    - Bit orientation when decoding `src/core/font.inl` (LSB/MSB) may be inverted.
    - Trim logic for variable-width glyphs (`fixed=false`) may misalign the leftmost drawn column vs expected TIC-80 behavior.
    - Row baseline/advance constants (ADV vs actual TIC font metrics) could be off by one.
  - Plan: Verify bit order against TIC font reference, ensure left trim maps the first non-empty glyph column to `x`, confirm row 0 draws the top row of the glyph, adjust ADV/height semantics as needed.

**Near-Term Backlog**
- Fix `lua_print_defaults_and_pix_read` by aligning `print_text` to TIC-80 semantics.
- Add unit tests for `print_text` width, left trim, and origin pixel behavior (non-Lua) to localize failures.
- Expose `clip` and `rectb` to Lua; add basic tests.
- Confirm palette correctness end-to-end (index→RGBA) against known swatches; keep golden samples.

**Mid-Term Backlog**
- Flesh out `tic-gfx` primitives (`circb/circ/elli/ellib/tri/trib/font/map/spr`) per `docs/api_parity_checklist.md`.
- Start `tic-api` facade layering (separating binding from core) as we add more APIs.
- Add frame-hash snapshot tests for deterministic VRAM state.
- Begin input semantics (`btn/key/keyp/btnp/mouse`) with fixed-step repeat behavior.

**Open Questions**
- Exact bit orientation for `font.inl` (confirm LSB/MSB and row order vs. current implementation).
- Default `print` metrics: width advance, left/right trimming, and baseline rules in TIC-80.
- Small font (`smallfont=true`) parity requirements and when to implement.

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

**Docs Index**
- Start here: `docs/README.md`
- Roadmap: `docs/roadmap/overview.md`, `docs/roadmap/gui_first.md`
- Specs: `docs/specs/memory_map.md`, `docs/specs/lua_api_parity.md`, `docs/specs/graphics.md`, `docs/specs/audio_fft_vqt.md`
- Architecture: `docs/architecture/workspace.md`, `docs/architecture/runtime.md`
- Testing: `docs/testing/strategy.md`, `docs/testing/frame_hashes.md`
- ADRs: `docs/adr/0001-winit-pixels.md`, `docs/adr/0002-mlua-lua54-compat.md`
