# TODO — Screenshot Feature

- [x] Write plan (`docs/architecture/screenshot_plan.md`).
- [x] Update README with CLI and usage.
- [x] Add `png` helper (`util::image`) with `save_png_rgba` and `scale_rgba_nn`.
- [x] CLI flags: `--screenshot`, `--screenshot-scale`, `--headless`, `--screenshot-frame`.
- [x] Windowed capture: implement one-off capture and F12 hotkey.
- [x] Headless capture: render offscreen and save; support `--editor` and cart run (up to frame N).
- [x] Unit tests: scaling + PNG encode/decode round-trip.
- [x] Integration smoke: headless render + save to a temp file.
- [x] Polish: create `./screenshots` dir and unique names; friendly errors.
- [x] Docs: add “Screenshots” examples to README and Test Catalog.
