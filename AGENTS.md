**TIC-80 Rust Rewrite — Agent Log**

- **Owner:** AI Coding Agent (Codex CLI)
- **Scope:** Help drive the Rust rewrite forward with tests-first changes, targeted features, and tight parity with TIC-80 behavior.
- **Last Updated:** 2025-08-27

**Documentation Discipline — Agent Reminder (PROMINENT)**
- ALWAYS document code changes immediately after writing code:
  - Update this file’s Current Status and Worklog (with date + concise bullets).
  - Update relevant docs (specs/architecture/testing) and cross-link from here.
  - Update `docs/specs/implementation_status.md` to reflect new capabilities.
  - If the plan changed, update the plan doc(s) and link them (Roadmap/Spec).
- ALWAYS document plans as first-class docs:
  - Add a plan/Implementation TODOs section under the relevant spec (e.g., audio FFT/VQT) or create a new spec.
  - Reference new/updated plans from AGENTS.md and the docs index.
- Keep hygiene visible: mention clippy/test status with each change.
- ALWAYS format code with `cargo fmt` after changes, in addition to fixing all compiler/clippy warnings and errors.
- ALWAYS update the test carts catalog when adding a new cart:
  - Add the cart to `docs/testing/test_carts.md` with purpose, run instructions, and expected behavior.
  - If needed, add a short run snippet to `docs/README.md`.
- Maintain a rolling TODO list from reviews in `docs/roadmap/todos_code_review.md` and tick items as they’re completed.

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
- Audio capture foundation: `cpal` input stream with mono downmix into a lock‑free ring buffer (8192 samples); CLI flags `--list-audio`, `--audio-device`, `--audio-vu`, `--audio-disable`; simple VU feedback prints peak dBFS once per second.
 - FFT: 2k R2C (`realfft`) on tick thread; maintains raw/smoothed/normalized buffers with peak tracking; `--debug-fft` throttled print; Lua `fft/ffts/fftr/fftrs` wired with C-identical clamping/sum semantics; headless tests added; simple cart at `assets/fft_test.lua`.
- Screenshots: CLI supports `--screenshot <path> [--screenshot-scale N] [--screenshot-frame N]` and `--headless` offscreen capture. In windowed mode, F12 saves to `./screenshots/scr-YYYYmmdd-HHMMSS.png` without exiting.
 - Editor: CODE view supports basic editing (insert chars/newline/tab, backspace/delete, Home/End), selection (Shift+arrows), clipboard (Ctrl/Cmd+C/V/X), undo/redo (Ctrl/Cmd+Z / Shift+Z or Y). Caret + auto-scroll; gutter and rendering intact; tests added.

**Near-Term Backlog**
- FFT implementation (cpal + realfft) per `docs/specs/audio_fft_vqt.md` (see Implementation TODOs section); add headless tests and Lua `fft/ffts/fftr/fftrs`.
- Livecoding editor (UI plan): Implement `tic-studio` per `docs/roadmap/editor_livecoding.md` — deliver CODE first, then CONSOLE (console scope may be reduced); TIC‑80 skin in framebuffer; hot reload; .tic code‑only round‑trip.
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
- Analysis (VQT): implement kernels + unwhitened/whitened paths per `docs/specs/audio_fft_vqt.md` (see Implementation TODOs) and expose `vqt*`/`vqt*w` APIs; conformance carts + numeric tolerances.
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
 - 2025-08-27:
   - Screenshots completed:
     - CLI flags implemented: `--screenshot`, `--screenshot-scale`, `--screenshot-frame`, `--headless`.
     - Windowed one-off capture wired; exits after saving when path is provided.
     - F12 hotkey saves to `./screenshots/scr-YYYYmmdd-HHMMSS.png` (auto-creates directory).
     - Headless renderer: draws editor UI/code view or runs cart ticks offscreen, then saves.
     - Tests: added `tests/screenshot_smoke.rs` to verify scaled PNG dimensions; extended util image tests.
     - Hygiene verified: `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings` clean; `cargo test` all green.
   - Implemented audio capture with `cpal`: device selection/listing, mono downmix, ring buffer.
   - Added CLI: `--list-audio`, `--audio-device`, `--audio-vu`, `--audio-disable`.
   - Integrated a 1s VU peak readout for manual verification.
   - Implemented 2k FFT analysis (realfft) with normalized/smoothed buffers and debug print flag; wired Lua FFT APIs; added headless tests and an FFT test cart.
   - Kept clippy/tests green; documented the FFT/VQT plan and linked TODOs.
   - Added clippy policy doc (docs/architecture/clippy_policy.md) and linked in docs index.
   - Refactored `main.rs` into helpers: args parsing, window/pixels init, audio init, device listing, script load. Behavior unchanged; code easier to lint/extend.
   - Error handling improvements:
     - Lua init errors now print once; warn once when cart defines no TIC().
     - BOOT() and TIC() call errors are logged to console instead of being dropped.
     - FFT/VQT update now warns once if realfft processing fails.
   - Audio capture robustness:
     - Selects nearest supported sample rate to requested (default 44100 Hz) and logs the choice.
     - Added ring buffer counters (pushes/overflows) and consumer underrun + consumed counters; `--debug-fx` prints per‑second deltas and totals with avg samples/tick and estimated occupancy.
   - Memory map clarity: replaced magic numbers with named constants; documented screen nibble packing.
   - Bit manipulation tests: added round-trip tests for 1/2/4/8‑bit ops, VRAM screen boundary, and unaligned 4‑bit sequences (nibble order).
   - Lua runner quiet mode: added `--quiet` flag and a global switch; suppresses once-only init/missing TIC and TIC/BOOT error prints for headless runs.
   - Editor (Phase 0 + Phase 1 partial):
     - Added `--editor` flag to launch a framebuffer UI with TIC‑80‑style chrome and tabs; top‑bar buttons auto‑size and center labels.
     - CODE view: rope‑backed buffer (ropey), read‑only viewport rendering, gutter with line numbers, arrow‑key caret navigation with auto‑scroll.
     - Caret: TIC‑style red box with 1px drop shadow; glyph under caret drawn dark to simulate inversion; aligned to a true 6×8 cell grid (fixed‑width glyphs render only left 6 columns, advance 6 px).
     - Tests: editor smoke (draw + tab switch), code viewport rendering (gutter/text pixels).
     - Kept clippy/tests green.
   - Verified hygiene: `cargo clippy --all-targets --all-features -D warnings` is clean; `cargo test` all green; ran `cargo fmt`.
   - Tightened clippy setup (pedantic/nursery/cargo) with pragmatic allows:
     - Crate-level allows for TIC-style APIs and numeric DSP: `many_single_char_names`, `too_many_arguments`, `similar_names`, selected numeric cast lints, and multiple crate versions.
     - Module/function allows where appropriate: `too_many_lines`, `items_after_statements`, `missing_errors_doc`, `needless_pass_by_value`, `redundant_clone`, `option_if_let_else` refactors or allows where needed.
     - Fixed numerous lints in code (lossless casts via `From`, `mul_add`/`hypot`, `map_or(_else)`, format arg inlining, moved inner items to top of scopes).
     - Added small docs and `#[must_use]` on relevant fns; marked a few helpers `const` where safe.
     - Cargo metadata filled in to silence cargo_common_metadata; clippy now passes with `-D warnings` across all targets.
   - Ran `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings`, and `cargo test`: all green.
 - Conducted a full code review of the `tic80_rust` crate. Findings are positive; suggestions for minor refactorings have been logged in `docs/roadmap/todos_code_review.md` and a summary added to `docs/adr/codereviews/001.md`.
 - Performed a second code review. The summary is located at `docs/adr/codereviews/002_ai_review.md` and actionable suggestions are in `docs/roadmap/todos_from_ai_review.md`.
- 2025-08-27 (cont.):
   - Editor basic editing implemented and tested:
     - Text input (ReceivedCharacter), Enter newline, Tab → one space; Backspace/Delete; Home/End.
     - Selection with Shift+arrows; clipboard shortcuts (copy/cut/paste) using OS clipboard; select-all.
     - Undo/redo stacks with batching for replace; redo semantics fixed and tested.
     - Key handling wired in windowed path; caret and auto-scroll preserved.
     - Tests: `tic80_rust/tests/editor_editing_tests.rs`, `tic80_rust/tests/editor_selection_undo_tests.rs`.
     - Hygiene verified: `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` all green.
   - Editor polish and bugfixes:
     - macOS Cmd shortcuts fixed by using per-event modifiers (`KeyboardInput.modifiers`) for Cmd/Ctrl detection.
     - Selection highlight aligned with caret box (vertical off-by-one vs clip corrected); added unit test for alignment.

**Docs Index**
- Start here: `docs/README.md`
- Roadmap: `docs/roadmap/overview.md`, `docs/roadmap/gui_first.md`
- Specs: `docs/specs/memory_map.md`, `docs/specs/lua_api_parity.md`, `docs/specs/graphics.md`, `docs/specs/audio_fft_vqt.md`
 - Architecture: `docs/architecture/workspace.md`, `docs/architecture/runtime.md`, `docs/architecture/screenshot_plan.md`
- Testing: `docs/testing/strategy.md`, `docs/testing/frame_hashes.md`
- ADRs: `docs/adr/0001-winit-pixels.md`, `docs/adr/0002-mlua-lua54-compat.md` (superseded), `docs/adr/0003-lua53-with-compat.md`
