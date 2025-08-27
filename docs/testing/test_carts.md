# Test Carts (Manual Verification)

This page lists small Lua carts in `tic80_rust/assets/` intended for quick, manual checks of subsystems.

- `tic80_rust/assets/default.lua`
  - Purpose: Graphics primitives demo (cls, pix, line, rect/rectb, circ/circb, elli/ellib, tri/trib, clip, print) with gentle animation.
  - How to run:
    - `cargo run --manifest-path tic80_rust/Cargo.toml` (default cart), or
    - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/default.lua`
  - Expected: Crosshair, shapes, clipping box toggling every few seconds, and sprinkled “stars”.

- `tic80_rust/assets/alt.lua`
  - Purpose: Minimal cart to validate `cls`, `rect`, `pix` origin behavior.
  - How to run:
    - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/alt.lua`
  - Expected: Background fill, small filled square at origin, and a single marker pixel at (0,0).

- `tic80_rust/assets/fft_test.lua`
  - Purpose: Visualize normalized FFT bins as 32 bars.
  - How to run:
    - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/fft_test.lua --audio-device "<name-substr>"`
    - Optional: `--audio-vu` to print a 1s peak; `--debug-fft` to print the first 16 bins (throttled).
  - Expected: Bars respond to audio; silence trends to near-zero. VU ~-180 dBFS with silence.

- `tic80_rust/assets/time_trace_test.lua`
  - Purpose: Exercise `time()` and `trace()` APIs.
  - How to run:
    - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/time_trace_test.lua`
  - Expected: On-screen elapsed ms text; a small marker toggles color every second; console prints `sec=<n>` lines via `trace()`.

- `tic80_rust/assets/vqt_test.lua`
  - Purpose: Visualize VQT across 12 octaves (120 bins), with auto-toggle between unwhitened and whitened views.
  - How to run:
    - `cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/vqt_test.lua --audio-device "<name-substr>"`
  - Expected: 120 bars (2px each) filling the 240px width; octave grid lines at every 12 bins; legend shows current mode (`raw` vs `whitened`) toggled every ~3 seconds; peak bin outlined.

Notes
- These carts are designed for fast feedback during local development. They complement headless tests and can reveal platform quirks (devices, timing).
- Keep carts small, single-purpose, and deterministic where possible.
