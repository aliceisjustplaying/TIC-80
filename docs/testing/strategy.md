# Testing and Validation Strategy

**Layers**
- **Framebuffer unit tests:** Validate drawing primitives (`cls`, `pix`, `line`, `rect`, `rectb`), clipping, palette → RGBA mapping, and OOB behavior.
- **Lua bridge tests:** Verify exposed APIs (`cls/pix/line/rect/print/rectb/clip`) and lifecycle (`BOOT`/`TIC`), and that Lua semantics (e.g., OOB `pix` read returns `nil`) are preserved.
- **Deterministic snapshots:** Compute a simple hash of the 240×136 palette-index framebuffer (per tick) to assert determinism and enable future golden comparisons.
- **Headless E2E (CLI):** Invoke the binary with `--headless --screenshot` to generate PNGs and decode them to validate success and dimensions.
- **Audio blocks (later):** Hash mixed audio blocks with tolerance windows for FP differences.
- **Conformance carts (later):** Automate selected carts (headless) and compare traces/hashes to baselines.
- **Fuzzing (later):** Fuzz `.tic` loader and selected APIs for robustness.

**Deterministic Frame Hash**
- We use an FNV‑1a hash over palette indices for the entire 240×136 buffer.
- Rationale: cheap, stable across platforms, and independent of presentation (RGBA conversion is not hashed).
- Helper pseudocode (Rust):
  
  ```rust
  fn fb_hash(fb: &mut Framebuffer) -> u64 {
      let (w, h) = dimensions();
      let mut hash: u64 = 0xcbf29ce484222325; // FNV offset
      const FNV_PRIME: u64 = 0x00000100000001B3;
      for y in 0..(h as i32) {
          for x in 0..(w as i32) {
              let b = fb.pix(x, y, None).unwrap_or(0);
              hash ^= b as u64;
              hash = hash.wrapping_mul(FNV_PRIME);
          }
      }
      hash
  }
  ```

**What We Test Today**
- Framebuffer
  - **cls/pix:** Fill/reads; OOB reads return `None`; writes masked to 0..15.
  - **rect (fill):** In-bounds, clipping to viewport and active clip rectangle.
  - **rectb (border):** One‑pixel perimeter; clipped to viewport and active clip.
  - **line:** Endpoints and pixel counts for axis-aligned and sloped lines; robustness with far‑OOB endpoints.
  - **clip:** Set/reset and enforcement in `set_pixel`, `rect`, `rectb`, and `pix` write mode.
  - **palette blit:** Index→RGBA conversion correctness for selected samples.
- Lua bridge
  - **API wiring:** `cls/pix/line/rect/print/rectb/clip` exposed and working.
  - **OOB semantics:** `pix` OOB read returns `nil` in Lua.
  - **File loading:** Alternate cart (`assets/alt.lua`) loads and runs via in-memory string.
  - **Determinism:** Default cart’s frame hash is stable for N ticks; different tick counts produce different hashes.

See `docs/testing/test_catalog.md` for the current test list and intent.

**How To Run**
- Unit + Lua tests: `cd tic80_rust && cargo test`
- Clippy (treat warnings as errors): `cd tic80_rust && cargo clippy --all-targets --all-features -D warnings`
- E2E screenshots only: `cd tic80_rust && cargo test -q e2e_headless_cli`

**Future Additions**
- Expand primitives coverage (`circb/circ/elli/ellib/tri/trib`, `print` edge cases: scale>1 areas, baseline/advance, small font).
- Add frame-hash goldens for selected demo sequences (stable seeds and scripts).
- Introduce error-path tests for Lua type/arity mismatches and unknown APIs.
- Add input semantics tests (`key/keyp/btn/btnp/mouse`) with fixed-step repeat timing.
