#![allow(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    clippy::suboptimal_flops,
    clippy::imprecise_flops,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
use realfft::{num_complex::Complex, RealFftPlanner, RealToComplex};

pub struct VqtKernel {
    pub indices: Vec<usize>,
    pub real: Vec<f32>,
    pub imag: Vec<f32>,
}

pub struct VQTState {
    // Config
    n: usize,         // 8192
    half: usize,      // n/2
    sample_rate: u32, // Hz (usually 44100)
    bins: usize,      // 120

    // Rolling mono buffer (last samples)
    buf: Vec<f32>,
    write_idx: usize,
    filled: bool,

    // FFT planning and work buffers
    r2c: std::sync::Arc<dyn RealToComplex<f32>>,
    scratch: Vec<Complex<f32>>, // realfft 3.x uses Complex scratch
    input: Vec<f32>,
    spectrum: Vec<Complex<f32>>, // length half+1

    // Kernels (sparse frequency-domain)
    kernels: Vec<VqtKernel>,

    // Output buffers
    pub vqt_raw: Vec<f32>,
    pub vqt_sm: Vec<f32>,
    pub vqt_norm: Vec<f32>,
    // Peak normalization for raw path
    pub vqt_peak: f32,

    // Whitened copies
    pub vqt_w_raw: Vec<f32>,
    pub vqt_w_sm: Vec<f32>,
    pub vqt_w_norm: Vec<f32>,
    // Peak normalization for whitened path
    pub vqt_w_peak: f32,

    // Scratch buffers (pre-allocated to avoid per-update allocations)
    logm: Vec<f32>,
    env: Vec<f32>,

    // Error visibility (warn once if FFT processing fails)
    warned_fft_error: bool,
}

const VQT_BINS: usize = 120;
const VQT_MIN_FREQ: f32 = 19.445; // D#0/Eb0
const VQT_SMOOTHING_FACTOR: f32 = 0.3;
const VQT_PEAK_SMOOTH: f32 = 0.99;
const VQT_PEAK_MIN: f32 = 0.0001;

// Whitening params
const VQT_WHITEN_WIDTH: usize = 21; // odd
const VQT_WHITEN_ALPHA: f32 = 0.95;
const VQT_WHITEN_EPS: f32 = 1e-6;

impl VQTState {
    #[must_use]
    pub fn new(sample_rate: u32, rolling_capacity: usize) -> Self {
        let n = 8192usize;
        let half = n / 2;
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c = planner.plan_fft_forward(n);
        let input = r2c.make_input_vec();
        let spectrum = r2c.make_output_vec();
        let scratch = r2c.make_scratch_vec();

        let mut s = Self {
            n,
            half,
            sample_rate,
            bins: VQT_BINS,
            buf: vec![0.0; rolling_capacity.max(n)],
            write_idx: 0,
            filled: false,
            r2c,
            scratch,
            input,
            spectrum,
            kernels: Vec::with_capacity(VQT_BINS),
            vqt_raw: vec![0.0; VQT_BINS],
            vqt_sm: vec![0.0; VQT_BINS],
            vqt_norm: vec![0.0; VQT_BINS],
            vqt_peak: VQT_PEAK_MIN,
            vqt_w_raw: vec![0.0; VQT_BINS],
            vqt_w_sm: vec![0.0; VQT_BINS],
            vqt_w_norm: vec![0.0; VQT_BINS],
            vqt_w_peak: VQT_PEAK_MIN,
            logm: vec![0.0; VQT_BINS],
            env: vec![0.0; VQT_BINS],
            warned_fft_error: false,
        };
        s.generate_kernels();
        s
    }

    fn center_frequencies(&self) -> Vec<f32> {
        // Semitone steps from base
        (0..self.bins)
            .map(|i| VQT_MIN_FREQ * (2.0f32).powf(i as f32 / 12.0))
            .collect()
    }

    fn variable_q(center: f32) -> f32 {
        // Port of C schedule optimized for 8k
        if center < 25.0 {
            7.4
        } else if center < 30.0 {
            9.2
        } else if center < 40.0 {
            11.5
        } else if center < 50.0 {
            14.5
        } else if center < 65.0 {
            16.0
        } else if center < 160.0 {
            17.0
        } else if center < 320.0 {
            15.0
        } else if center < 640.0 {
            13.0
        } else {
            11.0
        }
    }

    fn adaptive_threshold(center: f32) -> f32 {
        let q = Self::variable_q(center);
        if q > 30.0 {
            0.005
        } else if q > 20.0 {
            0.01
        } else {
            0.02
        }
    }

    fn generate_kernels(&mut self) {
        let centers = self.center_frequencies();
        self.kernels.clear();
        for &f0 in &centers {
            let q = Self::variable_q(f0);
            let mut win_len = (q * (self.sample_rate as f32) / f0).round() as usize;
            if win_len > self.n {
                win_len = self.n;
            }
            if win_len < 32 {
                win_len = 32;
            }
            // Time-domain kernel placed centrally in full N-length buffer
            let mut time = vec![0.0f32; self.n];
            let start = (self.n - win_len) / 2;
            // Hamming window
            for i in 0..win_len {
                let w = 0.54
                    - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (win_len as f32 - 1.0)).cos();
                // Modulate with cosine (real part of complex exponential); center around N/2
                let idx = start + i;
                let phase =
                    2.0 * std::f32::consts::PI * f0 * ((idx as f32) - (self.n as f32 / 2.0))
                        / (self.sample_rate as f32);
                time[idx] = w * phase.cos();
            }
            // Normalize by window length
            for t in &mut time {
                *t /= win_len as f32;
            }
            // FFT to frequency domain
            let mut spec = self.r2c.make_output_vec();
            // reuse input buffer
            self.input[..self.n].copy_from_slice(&time);
            let mut scratch = self.r2c.make_scratch_vec();
            if let Err(e) = self
                .r2c
                .process_with_scratch(&mut self.input, &mut spec, &mut scratch)
            {
                if !self.warned_fft_error {
                    eprintln!("VQT kernel FFT error: {e}");
                    self.warned_fft_error = true;
                }
                continue;
            }

            // Build sparse kernel by thresholding magnitude
            let thr = Self::adaptive_threshold(f0);
            let mut idxs = Vec::new();
            let mut reals = Vec::new();
            let mut imags = Vec::new();
            for (k, c) in spec.iter().enumerate().take(self.half + 1) {
                let mag = (c.re * c.re + c.im * c.im).sqrt();
                if mag > thr {
                    idxs.push(k);
                    reals.push(c.re);
                    imags.push(c.im);
                }
            }
            self.kernels.push(VqtKernel {
                indices: idxs,
                real: reals,
                imag: imags,
            });
        }
    }

    pub fn ingest(&mut self, sample: f32) {
        self.buf[self.write_idx] = sample;
        self.write_idx += 1;
        if self.write_idx >= self.buf.len() {
            self.write_idx = 0;
            self.filled = true;
        }
    }

    fn copy_latest_window(&mut self) {
        let n = self.n;
        let len = self.buf.len();
        let end = self.write_idx;
        let start = (len + end).saturating_sub(n) % len;
        if start + n <= len {
            self.input[..n].copy_from_slice(&self.buf[start..start + n]);
        } else {
            let first = len - start;
            self.input[..first].copy_from_slice(&self.buf[start..]);
            self.input[first..n].copy_from_slice(&self.buf[..(n - first)]);
        }
    }

    pub fn update(&mut self) {
        if !self.filled && self.write_idx < self.n {
            return;
        }
        self.copy_latest_window();
        if let Err(e) =
            self.r2c
                .process_with_scratch(&mut self.input, &mut self.spectrum, &mut self.scratch)
        {
            if !self.warned_fft_error {
                eprintln!("VQT update FFT error: {e}");
                self.warned_fft_error = true;
            }
            return;
        }

        // Apply kernels
        for (i, ker) in self.kernels.iter().enumerate() {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            for (j, &idx) in ker.indices.iter().enumerate() {
                if idx > self.half {
                    continue;
                }
                let c = self.spectrum[idx];
                let kr = ker.real[j];
                let ki = ker.imag[j];
                re += c.re * kr - c.im * ki;
                im += c.re * ki + c.im * kr;
            }
            let mag = (re * re + im * im).sqrt() * 2.0;
            self.vqt_raw[i] = if mag.is_finite() && mag >= 0.0 {
                mag
            } else {
                0.0
            };
        }

        // Smoothing and normalization (raw path)
        let a = VQT_SMOOTHING_FACTOR;
        let mut peak = 0.0f32;
        for i in 0..self.bins {
            self.vqt_sm[i] = self.vqt_sm[i].mul_add(a, self.vqt_raw[i] * (1.0 - a));
            if self.vqt_sm[i] > peak {
                peak = self.vqt_sm[i];
            }
        }
        if self.vqt_peak <= 0.0 {
            self.vqt_peak = VQT_PEAK_MIN;
        }
        if peak > self.vqt_peak {
            self.vqt_peak = peak;
        } else {
            self.vqt_peak = self
                .vqt_peak
                .mul_add(VQT_PEAK_SMOOTH, peak * (1.0 - VQT_PEAK_SMOOTH));
        }
        if self.vqt_peak < VQT_PEAK_MIN {
            self.vqt_peak = VQT_PEAK_MIN;
        }
        let norm = 1.0 / self.vqt_peak;
        for i in 0..self.bins {
            let mut v = self.vqt_sm[i] * norm;
            if v > 1.0 {
                v = 1.0;
            }
            if !v.is_finite() {
                v = 0.0;
            }
            self.vqt_norm[i] = v;
        }

        // Whitening path
        // log domain (reuse pre-allocated scratch)
        for (i, mslot) in self.logm.iter_mut().enumerate().take(self.bins) {
            let m = if self.vqt_raw[i].is_finite() && self.vqt_raw[i] >= 0.0 {
                self.vqt_raw[i]
            } else {
                0.0
            };
            *mslot = (m + VQT_WHITEN_EPS).ln();
        }
        // moving average envelope
        let halfw = VQT_WHITEN_WIDTH / 2;
        for i in 0..self.bins {
            let start = i.saturating_sub(halfw);
            let end = (i + halfw).min(self.bins - 1);
            let mut sum = 0.0f32;
            let mut count = 0;
            for val in self.logm.iter().take(end + 1).skip(start) {
                sum += *val;
                count += 1;
            }
            self.env[i] = if count > 0 {
                sum / count as f32
            } else {
                self.logm[i]
            };
        }
        // whiten and mix
        for i in 0..self.bins {
            let wlog = self.logm[i] - self.env[i];
            let mut wamp = wlog.exp_m1();
            if !wamp.is_finite() || wamp < 0.0 {
                wamp = 0.0;
            }
            let raw = self.vqt_raw[i];
            let mut mixed = (1.0 - VQT_WHITEN_ALPHA).mul_add(raw, VQT_WHITEN_ALPHA * wamp);
            if !mixed.is_finite() || mixed < 0.0 {
                mixed = 0.0;
            }
            self.vqt_w_raw[i] = mixed;
        }
        // Smooth and normalize whitened
        let mut wpeak = 0.0f32;
        for i in 0..self.bins {
            self.vqt_w_sm[i] = self.vqt_w_sm[i].mul_add(a, self.vqt_w_raw[i] * (1.0 - a));
            if self.vqt_w_sm[i] > wpeak {
                wpeak = self.vqt_w_sm[i];
            }
        }
        if self.vqt_w_peak <= 0.0 {
            self.vqt_w_peak = VQT_PEAK_MIN;
        }
        if wpeak > self.vqt_w_peak {
            self.vqt_w_peak = wpeak;
        } else {
            self.vqt_w_peak = self
                .vqt_w_peak
                .mul_add(VQT_PEAK_SMOOTH, wpeak * (1.0 - VQT_PEAK_SMOOTH));
        }
        if self.vqt_w_peak < VQT_PEAK_MIN {
            self.vqt_w_peak = VQT_PEAK_MIN;
        }
        let wnorm = 1.0 / self.vqt_w_peak;
        for i in 0..self.bins {
            let mut v = self.vqt_w_sm[i] * wnorm;
            if v > 1.0 {
                v = 1.0;
            }
            if !v.is_finite() {
                v = 0.0;
            }
            self.vqt_w_norm[i] = v;
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn bins_count(&self) -> usize {
        self.bins
    }

    // Expose scratch buffer addresses/capacity (useful for tests and diagnostics).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn scratch_ptrs(&self) -> (*const f32, *const f32) {
        (self.logm.as_ptr(), self.env.as_ptr())
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn scratch_caps(&self) -> (usize, usize) {
        (self.logm.capacity(), self.env.capacity())
    }
}

use parking_lot::RwLock;
use std::sync::{Arc, OnceLock};
static VQT_SHARED: OnceLock<Arc<RwLock<VQTState>>> = OnceLock::new();

pub fn set_global_vqt(state: Arc<RwLock<VQTState>>) {
    let _ = VQT_SHARED.set(state);
}

pub fn get_global_vqt() -> Option<&'static Arc<RwLock<VQTState>>> {
    VQT_SHARED.get()
}

#[must_use]
pub fn query_vqt(state: &VQTState, bin: i32, smoothing: bool, whitened: bool) -> f64 {
    let Ok(i) = usize::try_from(bin) else {
        return 0.0;
    };
    if i >= state.bins_count() {
        return 0.0;
    }
    if !whitened {
        if smoothing {
            f64::from(state.vqt_norm[i])
        } else {
            // instantaneous normalized (raw divided by peak)
            f64::from(state.vqt_raw[i] / state.vqt_peak)
        }
    } else if smoothing {
        f64::from(state.vqt_w_norm[i])
    } else {
        f64::from(state.vqt_w_raw[i] / state.vqt_w_peak)
    }
}
