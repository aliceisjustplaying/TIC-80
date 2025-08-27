# Audio Analysis (FFT/VQT)

This page captures the runtime behavior and the implementation plan for FFT/VQT in the Rust rewrite. Canonical derivations and design notes still live in `CLAUDE.md` at the repo root; this page focuses on API semantics, platform choices, and test strategy for parity with the C implementation.

## Goals
- API Parity: Implement `fft/ffts/fftr/fftrs`, `vqt/vqts/vqtr/vqtrs`, and `vqtw/vqtsw/vqtrw/vqtrsw` with identical signatures and observable behavior.
- Determinism: Stable outputs given fixed inputs (headless tests feed synthetic signals).
- Performance: Desktop targets (Windows/macOS/Linux) at real‑time rates (2k FFT ~21 fps; 8k VQT ~5.4 fps).

## Technology Choices (Rust)
- Audio I/O: `cpal` for capture (pure Rust), f32 stereo → mono downmix in callback.
- FFT: `realfft` (R2C) for N=2048 and N=8192 transforms; built on `rustfft`.
- Buffering: `ringbuf` (lock‑free SPSC) between audio callback and analysis thread.
- Windows: `window-functions` or `apodize` for Hamming/Gaussian in VQT kernel generation.

## Sample Rate and Buffers
- Sample Rate: 44100 Hz throughout (resample to 44.1k if device provides a different rate; future work).
- Shared Audio Buffer: `AUDIO_BUFFER_SIZE = max(2*FFT_SIZE, VQT_FFT_SIZE) = 8192` mono f32 samples (aligned with C).
- Capture Format: accept i16/u16/f32; convert to f32, average L/R to mono; no allocations in the callback.

## FFT (2k) — Behavior and Plan
- Window: last 2048 mono samples (from the shared buffer).
- Transform: R2C length 2048; magnitude per bin: `2.0 * hypot(re, im)`.
- Bins: expose 0..1023 (drop Nyquist) to match C.
- Buffers (mirroring C):
  - Raw: `fftRawData[1024]`, `fftRawSmoothingData[1024]` (IIR smoothing factor 0.6).
  - Display: `fftData[1024]`, `fftSmoothingData[1024]`, `fftNormalizedData[1024]`.
  - Peak tracking: `fPeakMinValue=0.01`, `fPeakSmoothing=0.995`, `fPeakSmoothValue`, `fAmplification=1/peak`.
- Update (per tick):
  1) Read last 2048 samples; zero‑pad until warm.
  2) R2C → magnitudes; update raw and smoothed raw.
  3) Update peak smoothing, recompute amplification, write normalized and smoothed normalized.
- APIs:
  - `fft(start,end=-1)`: normalized value at `start` or inclusive sum over `[start..end]` with C’s clamping rules.
  - `ffts(start,end)`: smoothed normalized.
  - `fftr/fftrs`: raw and raw‑smoothed (no normalization).

## VQT (8k) — Behavior and Plan
- Window: last 8192 mono samples.
- Kernels (once):
  - Centers: 120 bins starting at 19.445 Hz (D#0/Eb0), semitone spacing `2^(1/12)`.
  - Variable Q: replicate C’s schedule; window length `Q * fs / f`, clamped to 8192; minimum length guard.
  - Windowing: Hamming or Gaussian (match current default); normalize by window length.
  - Modulation: complex exponential across full FFT buffer, centered at N/2 (parity with C notes and indices).
  - R2C kernel FFT; sparsify to `(indices[], real[], imag[])` above adaptive magnitude thresholds.
- Runtime VQT (per tick):
  1) R2C of input frame (8192).
  2) For each kernel: sparse complex dot over half‑spectrum (0..4096), magnitude then `* 2.0`.
  3) Unwhitened path:
     - Smooth `vqtSmoothingData` with `VQT_SMOOTHING_FACTOR=0.3`.
     - Peak smoothing to `vqtPeakSmoothValue` (0.99 mix as in C pattern), normalize to `vqtNormalizedData` in [0,1].
  4) Whitened path (separate arrays):
     - Whitening enabled by default: log(m+eps) → moving average over width=21 → subtract → exp → mix by `alpha=0.95`.
     - Smooth to `vqtWhiteSmoothingData`; separate peak tracker `vqtWhitePeakSmoothValue`; normalize to `vqtWhiteNormalizedData`.
- APIs (two parallel sets):
  - Unwhitened: `vqt` (normalized), `vqts` (smoothed+normalized), `vqtr` (raw), `vqtrs` (raw smoothed).
  - Whitened: `vqtw`, `vqtsw`, `vqtrw`, `vqtrsw` with identical semantics on whitened buffers.

## Parity Rules
- Bins and scaling match C exactly: drop Nyquist for 2k FFT; use `2.0` magnitude factor.
- Peak tracking and smoothing factors: FFT raw/display (0.6), peak (0.995); VQT smoothing (0.3), whitening width=21, alpha=0.95, eps=1e‑6.
- Inclusive sums and clamping follow C’s logic for out‑of‑bounds and reversed ranges.
- Whitening is strictly a separate path with its own buffers and peak tracker.

## Testing Strategy
- Headless tests feed synthetic tones/noise into the analysis (bypass `cpal`).
- FFT:
  - Single‑tone peak lands at expected bin; normalized steady state ≈ 1.0.
  - Raw vs smoothed differ per IIR; inclusive sum matches expected bin sums.
- VQT:
  - Kernel spot checks: indices non‑empty; ranges align with center frequencies.
  - Single‑tone hits expected semitone bin; whitened vs unwhitened differ predictably on broadband inputs.
  - Normalization clamps to [0,1]; whitened/unwhitened use independent peak trackers.

## Debugging & Telemetry
- `--debug-fft`: Prints the first 16 FFT bins (smoothed normalized) periodically (~500 ms) for sanity checks.
- `--debug-fx`: Prints average processing time per tick (milliseconds) for FFT and VQT once per second (uses tick-thread timings). Useful to monitor headroom and spot regressions.

## Integration Plan (Milestones)
1) FFT foundation: ring buffer, 2k R2C planner, raw/normalized/smoothed buffers, Lua `fft/ffts/fftr/fftrs`.
2) VQT kernels: generation + storage; 8k R2C planner; unwhitened path with `vqt/vqts/vqtr/vqtrs`.
3) Whitening path: `vqtw/vqtsw/vqtrw/vqtrsw` with separate smoothing/peaks; feature flag to toggle.
4) Cross‑platform validation: confirm capture paths (WASAPI/CoreAudio/ALSA/Pulse/JACK) and resampling if needed.

## References
- Canonical details: `CLAUDE.md`.
- C sources for parity: `src/ext/fft.c`, `src/fftdata.h/.c`, `src/ext/vqt.c`, `src/vqtdata.h/.c`, `src/ext/vqt_kernel.c`.


## Implementation TODOs

Status
- Phase 1 implemented: `cpal` capture + mono downmix + ring buffer + CLI flags + VU feedback.
- Phase 2 implemented: FFT 2k using `realfft` (tick‑thread), raw/smoothed/normalized buffers and peak tracking; Lua `fft/ffts/fftr/fftrs` wired (C-identical semantics), headless tests added; optional `--debug-fft`.

Phase 1: Audio (cpal)
- Device listing: Add `--list-audio` to print capture devices and default.
- Device selection: Add `--audio-device "<name-substr>"` to pick device by substring; default to system default input.
- Sample rate: Request 44100 Hz; if device differs, accept nearest and record actual rate; add `--audio-rate 44100` override (future resampling).
- Channel format: Accept f32/i16/u16; convert to f32; downmix stereo to mono by average.
- Buffering: Add SPSC ring buffer (capacity 8192 f32 mono samples). No allocations in callback.
- Threading: Start `cpal` input stream on app init; push samples into ring buffer; graceful start/stop.
- UX check: `--audio-vu` to print a simple VU meter/peak every second for manual verification.
- Errors: Clear messages on device open failures; fallback to default device if selection fails; allow `--audio-disable`.

Phase 2: FFT (2k)
- Planner: Initialize `realfft` R2C plan for N=2048; allocate scratch/output once.
- State buffers:
  - Raw: `fftRaw[1024]`, `fftRawSm[1024]` (IIR smoothing factor 0.6).
  - Display: `fftData[1024]`, `fftSm[1024]`, `fftNorm[1024]`.
  - Peaks: `fPeakMin=0.01`, `fPeakSmooth=0.995`, `fPeak`, `fAmpl=1/fPeak`.
- Tick update:
  - Read last 2048 mono samples (zero-fill until warm).
  - R2C; magnitudes for bins 0..1023 as `2.0 * hypot(re, im)`.
  - Update raw/smoothed raw; update `fPeak` and `fAmpl`; write normalized and smoothed normalized.
- Lua APIs: `fft/ffts/fftr/fftrs` with C-identical clamping and inclusive-sum semantics.
- Tests (headless): tone peak at expected bin; normalized steady-state ≈ 1; smoothed below raw; range sums; OOB clamping.
- Telemetry: Optional `--debug-fft` dump.

Phase 3: VQT (8k)
- Planner: Initialize `realfft` R2C plan for N=8192; allocate scratch/output once.
- Kernel generation (once):
  - Centers: 120 semitone bins from 19.445 Hz (`2^(1/12)` spacing).
  - Q schedule: Mirror C’s rules; window length `Q*fs/f`, clamp to 8192; min length guard.
  - Windowing: Hamming (default) and Gaussian; normalize by window length.
  - Modulation: Complex exponential over full 8192 buffer centered at N/2.
  - R2C of time kernel; sparsify to `(indices, real, imag)` with adaptive thresholds.
- VQT state buffers (unwhitened): `vqt[120]`, `vqtSm[120]` (0.3), `vqtNorm[120]`, `vqtPeak`.
- Tick update (unwhitened): R2C input; sparse complex dot per bin over 0..4096; magnitude `* 2.0`; smooth; peak-smooth; normalize to [0,1].
- Lua APIs: `vqt/vqts/vqtr/vqtrs`.
- Tests: kernel sanity (non-empty indices, expected spans); 440 Hz tone peak; normalization bounds.

Phase 4: Whitening (separate path)
- Config: Feature flag `whitening=true` (default); `--no-whitening` toggle.
- Whitening buffers: `vqtW[120]`, `vqtWSm[120]`, `vqtWNorm[120]`, `vqtWPeak`.
- Whitening math: log(m+eps) → moving-average env (width=21) → subtract → exp → mix by alpha=0.95.
- Tick update (whitened): smooth; peak-smooth; normalize to [0,1]; if disabled, mirror unwhitened into whitened buffers.
- Lua APIs: `vqtw/vqtsw/vqtrw/vqtrsw`.
- Tests: broadband input flatter in whitened vs unwhitened; toggle affects only whitened set.

Cross‑cutting
- Determinism: All analysis in tick thread; audio callback only pushes to ring.
- Performance: Reuse FFT plans/buffers; no alloc in hot paths; optional `--debug-fx` timing logs.
- Error handling: Safe fail if no audio device; tests bypass `cpal`.
- Docs: Keep this page as source of truth; update AGENTS.md Near-/Mid‑Term backlog as milestones are delivered.
