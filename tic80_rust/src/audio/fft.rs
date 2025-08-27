use parking_lot::RwLock;
use realfft::{num_complex::Complex, RealFftPlanner, RealToComplex};
use std::sync::{Arc, OnceLock};

pub struct FFTState {
    // Analysis window
    n: usize,    // 2048
    half: usize, // 1024
    // Rolling mono buffer (last samples), sized to at least n (we use 8192 for future VQT)
    buf: Vec<f32>,
    write_idx: usize,
    filled: bool,

    // FFT planning and work buffers
    r2c: std::sync::Arc<dyn RealToComplex<f32>>,
    scratch: Vec<Complex<f32>>,
    input: Vec<f32>,
    spectrum: Vec<Complex<f32>>, // length half+1

    // Output buffers (length half)
    pub fft_raw: Vec<f32>,    // magnitudes (2.0 * |X[k]|)
    pub fft_raw_sm: Vec<f32>, // smoothed raw (factor 0.6)
    pub fft_data: Vec<f32>,   // normalized magnitudes
    pub fft_sm: Vec<f32>,     // smoothed normalized

    // Peak tracking for normalization
    f_peak_min: f32,       // 0.01
    f_peak_smoothing: f32, // 0.995
    f_peak_value: f32,
    f_amplification: f32,

    // Smoothing factor for displayed series
    f_smooth_factor: f32, // 0.6
}

impl FFTState {
    pub fn new(rolling_capacity: usize) -> Self {
        let n = 2048usize;
        let half = n / 2;
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c = planner.plan_fft_forward(n);
        let input = r2c.make_input_vec();
        let spectrum = r2c.make_output_vec();
        let scratch = r2c.make_scratch_vec();
        Self {
            n,
            half,
            buf: vec![0.0; rolling_capacity.max(n)],
            write_idx: 0,
            filled: false,
            r2c,
            scratch,
            input,
            spectrum,
            fft_raw: vec![0.0; half],
            fft_raw_sm: vec![0.0; half],
            fft_data: vec![0.0; half],
            fft_sm: vec![0.0; half],
            f_peak_min: 0.01,
            f_peak_smoothing: 0.995,
            f_peak_value: 0.01,
            f_amplification: 1.0,
            f_smooth_factor: 0.6,
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
        // Copy last n samples ending at write_idx (exclusive)
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
        // Ensure we have at least one full window written once
        if !self.filled && self.write_idx < self.n {
            // Not enough data yet; keep buffers near zero
            return;
        }
        self.copy_latest_window();
        // Forward R2C
        self.r2c
            .process_with_scratch(&mut self.input, &mut self.spectrum, &mut self.scratch)
            .ok();

        // Magnitudes for 0..half-1 (drop Nyquist at index half)
        let mut peak_raw = self.f_peak_min;
        for k in 0..self.half {
            let c = self.spectrum[k];
            let mag = (c.re * c.re + c.im * c.im).sqrt() * 2.0;
            self.fft_raw[k] = mag;
            if mag > peak_raw {
                peak_raw = mag;
            }
        }

        // Update peak smoothing and amplification
        if peak_raw > self.f_peak_value {
            self.f_peak_value = peak_raw;
        } else {
            self.f_peak_value = self.f_peak_value * self.f_peak_smoothing
                + peak_raw * (1.0 - self.f_peak_smoothing);
        }
        if self.f_peak_value < self.f_peak_min {
            self.f_peak_value = self.f_peak_min;
        }
        self.f_amplification = 1.0 / self.f_peak_value;

        // Smoothed raw and normalized series
        let a = self.f_smooth_factor; // 0.6
        for k in 0..self.half {
            let raw = self.fft_raw[k];
            let raw_sm = self.fft_raw_sm[k] * a + raw * (1.0 - a);
            self.fft_raw_sm[k] = raw_sm;

            let norm = raw * self.f_amplification;
            let norm_sm = self.fft_sm[k] * a + norm * (1.0 - a);
            self.fft_data[k] = norm;
            self.fft_sm[k] = norm_sm;
        }
    }

    pub fn bins(&self) -> usize {
        self.half
    }
}

// Global shared FFT state for Lua API access.
static FFT_SHARED: OnceLock<Arc<RwLock<FFTState>>> = OnceLock::new();

pub fn set_global_fft(state: Arc<RwLock<FFTState>>) {
    let _ = FFT_SHARED.set(state);
}

pub fn get_global_fft() -> Option<&'static Arc<RwLock<FFTState>>> {
    FFT_SHARED.get()
}

// Helper function to query FFT arrays with C-like clamping semantics.
// smoothing=false => normalized (fft_data) or raw (fft_raw);
// smoothing=true  => normalized-smoothed (fft_sm) or raw-smoothed (fft_raw_sm).
pub fn query_fft(state: &FFTState, start: i32, end: i32, smoothing: bool, raw: bool) -> f64 {
    let size = state.half as i32; // 1024
    if end == -1 {
        if start < 0 || start >= size {
            return 0.0;
        }
        let idx = start as usize;
        let v = if raw {
            if smoothing {
                state.fft_raw_sm[idx]
            } else {
                state.fft_raw[idx]
            }
        } else if smoothing {
            state.fft_sm[idx]
        } else {
            state.fft_data[idx]
        };
        return v as f64;
    }
    // both out-of-bounds on same side => 0
    if (start < 0 && end < 0) || (start >= size && end >= size) {
        return 0.0;
    }
    let mut s = start;
    let mut e = end;
    if s < 0 {
        s = 0;
    }
    if s >= size {
        s = 0;
    }
    if e >= size {
        e = size - 1;
    }
    if s > e {
        e = s;
    }
    let mut sum = 0.0f64;
    for i in s..=e {
        let u = i as usize;
        let v = if raw {
            if smoothing {
                state.fft_raw_sm[u]
            } else {
                state.fft_raw[u]
            }
        } else if smoothing {
            state.fft_sm[u]
        } else {
            state.fft_data[u]
        } as f64;
        sum += v;
    }
    sum
}
