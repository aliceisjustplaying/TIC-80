use std::sync::Arc;

use parking_lot::RwLock;
use std::cell::RefCell;
use std::rc::Rc;
use tic80_rust::audio::fft::{query_fft, set_global_fft, FFTState};
use tic80_rust::core::memory::Memory;
use tic80_rust::gfx::framebuffer::Framebuffer;
use tic80_rust::script::lua_runner::LuaRunner;

fn gen_bin_exact_samples(k: usize, n_samples: usize) -> Vec<f32> {
    // Generate y[n] = sin(2π k n / N) with N=2048 to align with FFT bins
    let nfft = 2048.0f32;
    (0..n_samples)
        .map(|n| {
            let angle = 2.0 * std::f32::consts::PI * (k as f32) * (n as f32) / nfft;
            angle.sin()
        })
        .collect()
}

#[test]
fn fft_query_peak_at_bin() {
    let cap = 8192;
    let mut fft = FFTState::new(cap);
    let k = 64usize;
    let samples = gen_bin_exact_samples(k, 4096);
    for s in samples {
        fft.ingest(s);
    }
    fft.update();

    // raw magnitude at k should exceed neighbors
    let v_k = query_fft(&fft, k as i32, -1, false, true);
    let v_l = query_fft(&fft, (k as i32) - 1, -1, false, true);
    let v_r = query_fft(&fft, (k as i32) + 1, -1, false, true);
    assert!(
        v_k > v_l * 2.0 && v_k > v_r * 2.0,
        "expected distinct peak at bin k"
    );
}

#[test]
fn lua_fft_returns_normalized_bin() {
    // Prepare shared FFT state and feed a strong tone at bin k
    let cap = 8192;
    let fft = Arc::new(RwLock::new(FFTState::new(cap)));
    let k = 32usize;
    let samples = gen_bin_exact_samples(k, 4096);
    {
        let mut w = fft.write();
        for s in samples {
            w.ingest(s);
        }
        w.update();
    }
    set_global_fft(fft);

    // Lua script: mark pixel if normalized fft(k) > 0.5
    let script = format!(
        "\
        function BOOT() cls(0) end\n\
        function TIC()\n\
            local v = fft({k}, -1)\n\
            if v > 0.5 then pix(0,0,7) end\n\
        end\n",
        k = k
    );

    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    let runner = LuaRunner::new(fb.clone(), mem, &script).expect("lua init");
    runner.tick();

    assert_eq!(fb.borrow_mut().pix(0, 0, None), Some(7));
}

#[test]
fn fft_query_range_clamps_and_sums() {
    // Build an FFT state and fill normalized data with 1.0 for easy sums
    let mut fft = FFTState::new(8192);
    let size = fft.bins(); // 1024
    for k in 0..size {
        fft.fft_data[k] = 1.0;
        fft.fft_sm[k] = 1.0;
        fft.fft_raw[k] = 1.0;
        fft.fft_raw_sm[k] = 1.0;
    }

    // both below zero -> 0
    assert_eq!(query_fft(&fft, -10, -1, false, false), 0.0);
    // both above size -> 0
    assert_eq!(
        query_fft(&fft, size as i32, (size as i32) + 10, false, false),
        0.0
    );
    // start < 0 clamps to 0; end 5 => 6 bins
    assert_eq!(query_fft(&fft, -5, 5, false, false), 6.0);
    // start >= size clamps to 0; end 10 => 11 bins
    assert_eq!(query_fft(&fft, size as i32, 10, false, false), 11.0);
    // end >= size clamps to size-1; start 1020 => (1020..1023) length 4
    assert_eq!(
        query_fft(&fft, (size as i32) - 4, (size as i32) + 99, false, false),
        4.0
    );
    // start > end becomes start=end; single bin
    assert_eq!(query_fft(&fft, 10, 2, false, false), 1.0);
}
