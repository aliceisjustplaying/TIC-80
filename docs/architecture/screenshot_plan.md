# Screenshot Feature — Plan

Status
- Implemented in runner CLI and windowed modes; covered by unit + E2E tests.

Goals
- Enable saving a framebuffer screenshot during normal runs (hotkey) and when launched in headless mode.
- Provide deterministic, minimal-overhead capture for CI and debugging, with optional integer scaling.

CLI
- `--screenshot <path>`: capture once at first redraw (or selected frame) and exit.
- `--screenshot-scale <N>`: integer scale for output PNG (default 1).
- `--headless`: no window; render offscreen and save a screenshot.
- `--screenshot-frame <N>` (optional): capture after N frames/ticks, then exit.
- Hotkey: F12 saves to `./screenshots/scr-YYYYmmdd-HHMMSS.png` while running (does not exit).

Implementation Outline
- Add a tiny image helper (`util::image`):
  - `save_png_rgba(path, w, h, rgba)` using `png` crate.
  - `scale_rgba_nn(src, w, h, scale)` (nearest-neighbor integer scale).
- Windowed path:
  - Track a `ScreenshotState { path, scale, frame_target, saved }` in `main.rs`.
  - On `RedrawRequested`, after `fb.blit_to_rgba(frame)`, copy RGBA to Vec, optionally scale, save, and optionally exit.
  - F12 hotkey builds a timestamped path and saves immediately using the same helper.
- Headless path:
  - Initialize framebuffer and state without `winit + pixels`.
  - If `--editor`: render Editor UI + code viewport once; else load cart and run `TIC()` for `N` frames.
  - Convert framebuffer to RGBA, optionally scale, and save.

Testing
- Unit tests:
  - Verify `scale_rgba_nn` expands a small RGBA image correctly.
  - Encode to PNG in-memory and decode back to compare pixels.
- Integration smoke:
  - Headless render of a small scene (e.g., clear + draw) and save to a temp file; validate file existence and dimensions.
  - E2E CLI tests invoke the binary with `--headless --screenshot` and decode the output PNG.

Docs
- README CLI: add flags and a small “Screenshots” section with examples.
- Test Catalog: reference screenshot unit tests.
- Worklog (AGENTS.md): log the feature with brief UX.

Follow-ups
- `--screenshot-delay <ticks>` convenience when not using frames.
- Optionally embed metadata (frame/tick/flags) into PNG `tEXt`.
