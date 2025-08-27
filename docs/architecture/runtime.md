# Runtime Architecture

Fixed-Step Loop
- 60 FPS logical tick; present cadence may be vsync-locked or decoupled.
- Each tick: process input → run callbacks → render → present.

Callbacks
- `BOOT()` once after cart load; `TIC()` each frame; `SCN(row)`/`BDR(row)` during draw (later).
- Error policy: trace and stop or bubble to host; define consistent behavior per phase.

Presentation
- CPU-owned framebuffer as palette indices; `blit_to_rgba` maps to RGBA for window texture.
- Integer scaling to maintain crisp pixels.

Audio Capture
- `cpal` input stream runs in a platform callback and writes mono f32 samples into a lock‑free ring buffer sized to 8192.
- The analysis path (FFT/VQT) runs on the tick thread, reading from the ring’s tail to ensure determinism and keep heavy work off the audio thread.
