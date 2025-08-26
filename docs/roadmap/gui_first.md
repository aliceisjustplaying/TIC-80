# GUI-First Track: Window + cls/pix and Lua Wiring

Objective
- Open a window (winit + pixels), present a 240×136 framebuffer with integer scaling, and exercise `cls/pix` end-to-end, then wire Lua callbacks.

Decisions
- Platform: `winit + pixels` (desktop first, portable to Web).
- Input/Audio (later): `winit` (kbd/mouse), `gilrs` (gamepad), `cpal` (audio).
- Scaling: integer nearest-neighbor (2x/3x/4x). Default 3x. Preserve aspect.
- Pixel format: u8 palette indices → RGBA via 16-color palette from memory map.

Milestone 1 — GUI + cls/pix (no Lua)
- Deliverables
  - Presenter renders a CPU-owned 240×136 index buffer via `pixels` with NN scaling.
  - Minimal gfx spec for `cls/pix` and palette mapping.
  - Verification: unscaled buffer hashes + micro-demos (alternating `cls`, crosshair via `pix`, palette grid).
- Tasks
  - Finalize FB spec and present path; set vsync policy.
  - Implement `cls/pix` and buffer→RGBA blit; add unit tests.
  - Write hash fixtures and demo scripts (host-driven).

Milestone 2 — Lua + cls/pix
- Deliverables
  - `mlua` runner: BOOT once, TIC per frame; expose `cls/pix`.
  - Error policy for unimplemented APIs; argument handling for `pix` read/write modes.
  - Verification: deterministic script exercising `cls/pix` yields known hashes.
- Tasks
  - Bind `cls/pix`; confirm numeric conversions and return values match TIC-80.
  - Add tests for callback flow and pixel state.

Risks & Mitigations
- Color differences: use exact sRGB bytes; disable filtering.
- Timing jitter: fixed 60 FPS tick; decouple present cadence if needed.
- Lua numeric semantics: be explicit about integer vs float at the boundary.

Next
- Extend primitives (`print`, `line`, `rect/rectb`), then `spr/map`, `clip`, and `vbank`.

