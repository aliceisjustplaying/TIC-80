# Code Review TODOs (Live list)

This list tracks follow-ups from the code review (see `docs/adr/codereviews/001.md`). Items are prioritized. We tick items off as they land.

## High Priority
- [x] Error handling and visibility
  - [x] Warn once when Lua runner is absent (main loop)
  - [x] Log errors from TIC() calls without crashing
  - [x] Log (warn once) if realfft processing fails in FFT/VQT
- [x] Audio capture robustness
  - [x] Select nearest supported sample rate to 44100 Hz when exact not available; log choice
  - [x] Add ring overflow counters and print under `--debug-fx`
- [x] VQT wiring parity tests
  - [x] Ensure `vqt` (instantaneous normalized) != `vqts` (smoothed normalized) on first update
  - [x] Same for `vqtw` vs `vqtsw`
- [x] Memory map clarity
  - [x] Replace remaining magic numbers with named constants + doc comments where applicable
- [x] Bit manipulation tests
  - [x] Property-based round-trip tests for `peek_bits/poke_bits` across 1/2/4/8-bit widths
  - [x] Region boundary tests around memory edges

## Medium Priority
- [ ] Linting and CI
  - [ ] Add stricter lint profile (pedantic/nursery/cargo + rust_2018_idioms) in CI
  - [ ] Allowlist noisy lints where appropriate
- [ ] Contributor and design docs
  - [ ] CONTRIBUTING.md (fmt/clippy/test, docs discipline, running carts)
  - [ ] ADR: Editor approach (CODE first, CONSOLE later, framebuffer UI)
  - [ ] WASM feasibility note (platform abstraction for audio/input)
- [ ] Benchmarks (Criterion)
  - [ ] Framebuffer ops and blit_to_rgba
  - [ ] Memory peek/poke variants
  - [ ] FFT (2k) + VQT (8k sparse dots)
- [ ] Integration tests
  - [ ] .tic round-trip: only code section changes
  - [ ] Hot reload gating: failing edit keeps last-good; recovery on fix
  - [ ] Console: trace()/error paths observed deterministically

## Lower Priority / Optional
- [ ] Optional analysis worker thread (feature-flag) with double-buffered FFT/VQT outputs
- [ ] Editor prep (UI skin specifics)
  - [ ] Confirm font/palette assets and integer scale steps
  - [ ] Lock initial keybinds (Save/Run/Find)
  - [ ] Define hit-testing rectangles for tabs/buttons in 240×136 space

Notes
- Keep docs/worklog disciplined: whenever an item is completed, tick it here, add a note in AGENTS.md Worklog, and update relevant specs.
