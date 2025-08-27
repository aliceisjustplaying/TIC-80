# Code Review TODOs (from AI Review 2025-08-27)

This is a prioritized list of actionable suggestions generated from the AI code review on 2025-08-27. The full summary can be found in `docs/adr/codereviews/002_ai_review.md`.

## Medium Priority

-   **[ ] [VQT] Consolidate VQT API implementation:** In `script/lua_runner.rs`, the 12 VQT-related Lua functions (`vqt`, `vqts`, `vqtr`, etc.) are implemented as separate closures. This creates significant code repetition.
    -   **Suggestion:** Refactor this into a single helper function or macro that takes query parameters (e.g., bin, smoothing, raw, whitened) to dispatch to the correct buffer in `VQTState`. This would improve maintainability.

-   **[ ] [main] Refine `AudioState` struct:** The `AudioState` struct in `main.rs` has accumulated numerous fields for debugging and statistics (`fx_fft_acc_ns`, `last_pushed`, `ema_samples_per_tick`, etc.).
    -   **Suggestion:** Group related statistics fields into dedicated sub-structs (e.g., `FxStats`, `RingBufferStats`). This would make `AudioState` cleaner and the associated logic more modular.

## Low Priority / Nitpicks

-   **[ ] [FFT/VQT] Clarify `get_global_*` pattern:** The use of `OnceLock` for the global `FFTState` and `VQTState` is a pragmatic solution for accessing audio data from single-threaded Lua callbacks.
    -   **Suggestion:** Add a small comment to the `FFT_SHARED` and `VQT_SHARED` static declarations explaining *why* this pattern is used (e.g., "Global state for easy access from Lua callbacks, which don't easily accommodate passing user data through C boundaries.").

-   **[ ] [main] Redundant `help` check:** In `main.rs`, the `if args.help` block appears twice consecutively. The second instance is unreachable.
    -   **Suggestion:** Remove the duplicate block.

-   **[ ] [docs] Add a diagram:** The documentation is excellent, but a high-level component diagram could enhance its accessibility.
    -   **Suggestion:** Consider adding a simple Mermaid diagram to `docs/architecture/runtime.md` showing the interaction between the main components (`main loop`, `winit`, `pixels`, `cpal`, `ring buffer`, `FFT/VQT State`, `LuaRunner`).
