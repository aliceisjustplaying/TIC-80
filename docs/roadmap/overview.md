# TIC-80 Rust Rewrite Plan

This document outlines a pragmatic, staged plan to reimplement TIC-80 in Rust. It focuses on maintaining cartridge/API compatibility, achieving deterministic behavior, and enabling incremental delivery with measurable checkpoints. It also details an initial track that targets Lua-only support first, then broadens to full parity.

## Goals and Scope

- Compatibility: Run existing `.tic` cartridges unmodified; preserve memory map, API semantics, timing, and constraints (240×136, 16-color palette, sprites, map, 4 channels, etc.).
- Determinism: Per-frame determinism for a given seed/input across platforms.
- Performance: Match or exceed C implementation at 60 FPS on desktop targets.
- Portability: Desktop first (macOS, Linux, Windows), then WebAssembly; keep console ports out-of-scope initially.
- Phased Delivery: Start with core runtime + Lua API, defer Studio editors.

## Non-Goals (Initial Phases)

- Rewriting the Studio editors (code/sprite/map/music/sfx UIs) in Phase 1–4.
- Supporting all languages up front; we start with Lua-only to validate the Rust core.
- Feature-creep beyond documented TIC-80 behavior unless explicitly called out as optional.

## Reference Material (from this repo)

- Architecture: `src/core/` (runtime, draw, sound, io), `src/api/` (language bindings), `src/studio/` (editors), `src/system/` (SDL/libretro/n3ds), `src/cart.c` (cartridge), `src/tic.[ch]`, `src/api/lua*.c`.
- Audio analysis: FFT/VQT covered in CLAUDE.md with precise specs and behavior.
- Build/deps: `CMakeLists.txt`, `cmake/*.cmake` (Lua vendored, SDL, zlib, etc.).

## Architectural Decomposition (Rust)

Crates (workspace):
- `tic-core`: Core VM state, memory map, timing, resource management, cartridge model, save states.
- `tic-gfx`: CPU rasterizer for all primitives (`pix`, `line`, `circ`, `rect`, `spr`, `map`, `clip`, palette ops), VRAM, and draw modes.
- `tic-audio`: PSG-style synth + mixer matching `src/core/sound.c`; later add capture for FFT/VQT.
- `tic-io`: Input abstraction (keyboard/mouse/gamepad), filesystem/cart IO, clipboard, time.
- `tic-api`: Language-agnostic API facade mirroring `api.h` semantics against `tic-core/gfx/audio/io`.
- `tic-lua`: Lua runtime embedding and API shims that expose `tic-api` to Lua scripts.
- `tic-fx`: FFT/VQT analysis (post-capture) with smoothing, peak normalization, whitening.
- `tic-sdl` (or `tic-winit`): Platform layer for windowing, GL or GPU context, audio IO, input events.
- `tic-runner`: CLI binary that loads `.tic`, runs headless or with window; no Studio.
- `tic-studio` (later): Editor UI reimplementation (e.g., egui or immediate-mode toolkit) when core stabilizes.

Rationale: keep concerns isolated, enable headless testing and cross-checking per crate.

## Key Compatibility Requirements

- Cartridge Format: Load/save `.tic` exactly (including zlib packing). Preserve bank semantics and PRO extensions (only when targeted).
- Memory Map: Mirror addresses, sizes, and behavior (RAM/VRAM/persistent). Do not expose Rust internals to scripts.
- API Semantics: Preserve function names, parameters, return values, side effects, and error behavior. Lua callback flow (`TIC()`, `BOOT()`, `SCN()`, `BDR()`, `MENU()`); outline parsing not required initially.
- Timing: 60 FPS tick by default; menu/console interactions deferred until Studio phase; ensure fixed-step logic to match C.
- FFT/VQT: Reproduce CLAUDE.md specs (buffer sizes, smoothing factors, auto-gain, whitening parameters) for visual parity.

## Cross-Cutting Concerns and Decisions

- Lua engine: Use `mlua` with bundled Lua 5.4 and `LUA_COMPAT_5_2` equivalent behavior to match current build (`LUA_COMPAT_5_2` is defined in cmake). Avoid LuaJIT for portability; revisit later for perf.
- Rendering backend: Start with software rasterizer in `tic-gfx` (deterministic, headless testable). Add a thin SDL2 or winit+pixels presentation layer later. Optional `wgpu` in a later phase.
- Audio IO: Use `cpal` for capture/output. Keep synth/tick deterministic in `tic-audio`. For capture, implement a lock-free ring buffer shared with `tic-fx`.
- Compression: Use `flate2`/`miniz_oxide` for zlib compatible packing/unpacking of carts.
- FFT/VQT: Use `realfft` (R2C) for 2k/8k transforms with exact binning and smoothing behavior; implement variable‑Q kernel generation per CLAUDE.md. See `docs/specs/audio_fft_vqt.md` for the Rust plan.
- Testing: Frame-hash snapshots for VRAM, audio block-level comparisons, API-level golden tests. Conformance carts from `demos/`.

## Phased Roadmap

Phase 0 — Discovery and Spec (no code, docs + harness prep)
- Inventory `api.h` and Lua binding coverage in `src/api/luaapi.c` and `src/api/lua.c`.
- Write API parity checklist (functions, signatures, side effects, edge cases). Annotate open questions.
- Map memory layout and constraints from `src/tic.h`, `src/core/*`, and CLAUDE.md into a concise spec.
- Identify platform dependencies to replace (SDL, zlib, audio) and select Rust crates.
- Output: Design doc + checklists; crate layout proposal; risk register.

Phase 1 — Cartridge + Core State (headless)
- Implement `.tic` parsing/loading/saving; banks, metadata, palettes, sprites, map, code segment(s).
- Define `tic-core` state, memory map, and fixed-step ticker. No rendering/audio yet.
- Add basic CLI to inspect carts and dump metadata (headless validation).
- Output: Can load and introspect `.tic`; deterministic tick progression without side effects.

Phase 2 — Lua Runtime + API Shim (headless)
- Embed Lua; load script from cart; call `BOOT()` then per-frame `TIC()`.
- Implement a minimal subset of API in `tic-api` with in-memory effects only: `time()`, `peek/poke` (scoped), `trace`, seed/rng.
- Add harness to run demo carts in headless mode and record API calls.
- Output: Runs simple scripts that don’t draw or play sound; deterministic logs.

Phase 3 — Graphics Primitives (software)
- Implement full raster pipeline in `tic-gfx` (CPU): palette, clip, `pix/line/circ/rect/tri`, text (`print`), sprites (`spr`), map (`map`), blitting rules.
- Hook `tic-api` drawing calls to `tic-gfx`; expose a window via `tic-sdl`/`tic-winit` for presentation only.
- Add VRAM frame hashing and image snapshots; compare against reference C build where feasible.
- Output: Visual carts render correctly; headless tests produce stable hashes.

Phase 4 — Input + Timing
- Keyboard/mouse/gamepad in `tic-io`; map to TIC-80 semantics; ensure edge cases (press/repeat/up/down).
- Verify fixed-step timing at 60 FPS, decoupling present frequency if needed.
- Output: Interactive carts playable; deterministic input handling under recorded streams.

Phase 5 — Audio Synth + Mixing
- Reimplement `src/core/sound.c` behavior: waveforms, SFX, music tracker playback, mixing; ensure bit-exact or perceptually equivalent output.
- Add audio output via `cpal`; implement recordable audio blocks for tests.
- Output: Audio carts sound correct; block hashes/stats match baselines.

Phase 6 — FFT/VQT Feature Parity
- Implement shared capture buffer; 2048-sample FFT (21 fps) and 8192-sample VQT path per CLAUDE.md.
- Implement smoothing, peak normalization, whitening with configurable macros equivalent.
- Expose APIs: `fft/ffts/fftr/fftrs/vqt/vqts/vqtr/vqtrs/vqtw/vqtsw/vqtrw/vqtrsw`.
- Output: Visual analyzers from `demos/` behave identically.

Phase 7 — Studio (Optional, staged)
- Recreate Studio UI gradually (console, code, sprite, map, sfx, music) using immediate-mode UI (e.g., egui) or SDL-rendered IMGUI-like.
- Defer complex UX to later; keep feature set close to original.
- Output: Usable integrated editor after core stabilizes.

Phase 8 — WebAssembly + Additional Platforms
- WASM target (via `wasm32-unknown-unknown` + `wasm-bindgen`/`wasm32-wasi`), web audio, canvas; sandboxed storage.
- Optional mobile ports later.

## Testing and Validation Strategy

- API Goldens: Unit tests per API call against known inputs (including edge cases and errors).
- Frame Hashes: Hash VRAM after each frame to produce stable signatures for demo carts.
- Audio Blocks: Hash mixed audio blocks; tolerance windows for FP differences.
- Conformance Carts: Automate running `demos/` carts headless, comparing output to reference traces.
- Differential Testing: Optional—run identical carts on original C build and Rust build; compare frame hashes and key metrics.
- Fuzzing: Fuzz `.tic` loader and selected APIs; validate against panics and UB.

## Risks and Mitigations

- Lua Semantics Drift: Differences across 5.2/5.3/5.4. Mitigate by enabling compat flags and authoring tests around table iteration, integer/float behavior, coroutines.
- Rendering Differences: Pixel-perfect behavior required. Begin with CPU rasterizer, codify exact blending rules.
- Audio Timing: Latency/jitter from host APIs. Use ring buffers and fixed block sizes; decouple from visual ticks.
- FFT/VQT Performance: 8k FFT + VQT kernel costs. Optimize with plan reuse, sparse kernels, and careful allocation.
- WASM Constraints: No native threads; audio timing quirks. Defer WASM until desktop parity.

## Milestone Criteria (Per Phase)

- Phase 1: `.tic` carts parse and round-trip; metadata and assets intact.
- Phase 2: Headless Lua scripts run; `TIC()` tick loop stable; minimal API callable.
- Phase 3: Visual primitives render; hashes match baselines for selected carts.
- Phase 4: Input tests pass; deterministic playback under recorded inputs.
- Phase 5: Audio tests pass; tracker playback validated.
- Phase 6: FFT/VQT demo carts behavior matches; numeric metrics within tolerances.
- Phase 7/8: Usability and platform acceptance criteria defined separately.

## Work Breakdown for Lua-Only Track (Fast Path)

1) Phase 0 deliverables (API checklist + confirm/align to MEMORY_MAP.md).
2) Cartridge loader + Lua script extraction only; ignore non-Lua carts initially.
3) Headless Lua VM with `BOOT/TIC` and a minimal `tic-api` surface (`time`, `trace`, RNG, `peek/poke` across documented regions).
4) Software `tic-gfx` with presentation via SDL/winit.
5) Input + audio in later increments.

## First Tangible Step

Create an API Parity Inventory and align with MEMORY_MAP.md (no code). Concretely:
- From `src/api.h`, `src/api/luaapi.c`, and `src/api/lua.c`, enumerate every API function exposed to Lua (name, signature, return, side effects, error cases, and which subsystem it touches).
- Use `MEMORY_MAP.md` as the authoritative memory layout reference; where needed cross-check with `src/tic.h`/`src/core/*` for any inconsistencies.
- Output one short document in the repo (`docs/specs/lua_api_parity.md`) plus a proposed Rust crate layout; reference `MEMORY_MAP.md` instead of duplicating it.
- This gives a precise contract to implement and test against and avoids early rework. It also makes it straightforward to decide Lua compatibility settings in Rust (`LUA_COMPAT_5_2`).

If you’d prefer a “code-adjacent” first step instead: scaffold the Rust workspace and empty crates with READMEs and CI that only builds the workspace (no runtime code), then add the two spec docs above before implementing anything.

## Open Questions to Resolve Early

- Exact Lua version expectations for cartridges (5.2 compatibility features relied upon?).
- Required bit-exactness for audio vs. perceptual equivalence.
- Which demos form the conformance baseline set and acceptable numeric tolerances.
- Preferred desktop windowing stack (SDL2 vs. winit) for long-term maintenance.

## Appendix: Mapping Guide (C → Rust)

- `src/core/` → `tic-core` (state/tick), `tic-gfx` (draw), `tic-audio` (synth/mix), `tic-io` (io/input)
- `src/api/` → `tic-api` (surface), `tic-lua` (language binding)
- `src/system/sdl/` → `tic-sdl` (platform layer) or `tic-winit`
- `src/cart.c`/`zip.c` → `tic-core` cart loader + `flate2`/`miniz_oxide`
- FFT/VQT files per CLAUDE.md → `tic-fx`

---

This plan aims to de-risk the rewrite by isolating subsystems, proving compatibility early with headless testing, and deferring UI complexities until the core is stable.

## GUI-First Kickoff (Fast Lane)

If early GUI feedback is a priority, we can interleave a minimal presentation path to get a window plus `cls`/`pix` quickly while keeping scope small and deterministic:

- Chosen platform layer: `winit` + `pixels` (pure Rust). Rationale: mature, cross-platform (desktop + WebAssembly via wgpu), simple pixel buffer presentation, no system SDL deps. We can keep an optional SDL2 backend later if needed.
- Audio + input crates to pair: `cpal` (audio output/capture), `gilrs` (gamepad), `winit` (keyboard/mouse), `arboard` (clipboard) as needed later.
- Framebuffer spec: 240×136 logical surface with 16-color palette; render to an RGBA buffer for the OS window using nearest-neighbor integer scaling (2x/3x/4x) and optional vsync.
- Minimal VRAM: Implement only screen memory and palette mapping from MEMORY_MAP.md; defer palette map, border color, and vbank switching until later.
- Earliest API subset: `cls(color)`, `pix(x,y[,color])`, and `print(...)` optional; expose to a stub demo runner (no Lua yet) to validate the raster pipeline.
- Then wire Lua: Load Lua script, call `BOOT()`/`TIC()`, and plumb `cls`/`pix` through the Lua binding to the framebuffer.
- Acceptance: Static test image hashes at 1x (unscaled), integer-scaling visual inspection at 3x, and a tiny script that alternates `cls()` colors and sets a few pixels.

Suggested steps (no code yet):
- Document the choice (SDL2 vs winit) and scaling strategy; define pixel format and palette conversion rules (index→RGBA via 16×RGB in MEMORY_MAP.md).
- Specify the minimal structs for framebuffer and the two API calls, and the main loop responsibilities (tick at 60 FPS, present at vsync or unlocked).
- Identify 2–3 micro-demos for visual verification (e.g., alternating `cls`, crosshair via `pix`, and a palette sweep).

See also: `docs/roadmap/gui_first.md` for concrete milestone tasks and success criteria for GUI + cls/pix and Lua + cls/pix.
