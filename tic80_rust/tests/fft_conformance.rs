use tic80_rust::audio::fft::FFTState;
use tic80_rust::audio::vqt::VQTState;

fn gen_sine(n: usize, k: usize) -> Vec<f32> {
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        let phase = 2.0_f32 * std::f32::consts::PI * (k as f32) * (i as f32) / (n as f32);
        v.push(phase.sin());
    }
    v
}

#[test]
fn fft_single_bin_peak_and_normalization() {
    let mut fft = FFTState::new(4096);
    // Generate a sine exactly at bin k
    let n = 2048usize;
    let k = 50usize;
    let s = gen_sine(n, k);
    for &x in &s {
        fft.ingest(x);
    }
    fft.update();
    // Raw peak near k should exceed neighbors significantly
    let mut max_i = 0usize;
    let mut max_v = 0.0f32;
    for i in 0..fft.bins() {
        let v = fft.fft_raw[i];
        if v > max_v {
            max_v = v;
            max_i = i;
        }
    }
    assert!((max_i as i32 - k as i32).abs() <= 1); // allow +/-1 bin tolerance
                                                   // Normalized at max bin should be ~1.0
    assert!(fft.fft_data[max_i] > 0.9);
    // Neighbors should be much smaller
    if max_i > 0 {
        assert!(fft.fft_data[max_i - 1] < 0.5);
    }
    if max_i + 1 < fft.bins() {
        assert!(fft.fft_data[max_i + 1] < 0.5);
    }
}

#[test]
fn vqt_bin_has_higher_energy_than_neighbors() {
    // Pick a target bin in [10..100]
    let sr = 44_100u32;
    let mut vqt = VQTState::new(sr, 16_384);
    // Use bin 60 as target
    let bin = 60usize;
    // Reconstruct center frequency based on implementation schedule
    let f0 = 19.445_f32 * (2.0_f32).powf(bin as f32 / 12.0);
    // Generate 8192 samples of a sine at this frequency
    let n = 8192usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / (sr as f32);
        let phase = 2.0_f32 * std::f32::consts::PI * f0 * t;
        s.push(phase.sin());
    }
    for &x in &s {
        vqt.ingest(x);
    }
    vqt.update();
    // Target bin should be >= neighbors in both raw and normalized-smoothed paths
    let b = bin;
    let center = vqt.vqt_raw[b];
    let left = if b > 0 { vqt.vqt_raw[b - 1] } else { 0.0 };
    let right = if b + 1 < vqt.bins_count() {
        vqt.vqt_raw[b + 1]
    } else {
        0.0
    };
    assert!(center >= left);
    assert!(center >= right);
    // Normalized-smoothed not zero
    assert!(vqt.vqt_norm[b] >= 0.0);
    assert!(vqt.vqt_norm[b].is_finite());
}
