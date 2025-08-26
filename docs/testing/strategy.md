# Testing and Validation Strategy

Layers
- API goldens: Unit tests per API including edge cases and errors.
- Frame hashes: Deterministic VRAM hashes per frame to compare against baselines.
- Audio blocks: Hash mixed audio buffers with tolerances for FP differences.
- Conformance carts: Automated headless runs comparing traces/hashes to reference.
- Fuzzing: `.tic` loader and selected APIs for robustness.

Notes
- Prefer headless tests with minimal dependencies.
- Keep baselines small and documented; record how they are generated.

