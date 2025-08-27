use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use parking_lot::RwLock;
use tic80_rust::audio::vqt::{set_global_vqt, VQTState};
use tic80_rust::core::memory::Memory;
use tic80_rust::gfx::framebuffer::Framebuffer;
use tic80_rust::script::lua_runner::LuaRunner;

fn vqt_center_freq(bin: usize) -> f32 {
    19.445f32 * (2.0f32).powf(bin as f32 / 12.0)
}

fn gen_sine(freq: f32, sample_rate: u32, n: usize) -> Vec<f32> {
    let sr = sample_rate as f32;
    (0..n)
        .map(|i| (2.0 * std::f32::consts::PI * freq * (i as f32) / sr).sin())
        .collect()
}

#[test]
fn vqt_peak_at_expected_bin() {
    let cap = 8192;
    let sr = 44_100;
    let mut vqt = VQTState::new(sr, cap);
    let bin = 48usize; // frequency near 19.445 * 2^(48/12) ~ 311 Hz
    let f0 = vqt_center_freq(bin);
    let samples = gen_sine(f0, sr, 10_000);
    for s in samples {
        vqt.ingest(s);
    }
    vqt.update();
    // Check normalized peak near bin and greater than neighbors
    let v = vqt.vqt_norm[bin];
    let vl = if bin > 0 { vqt.vqt_norm[bin - 1] } else { 0.0 };
    let vr = vqt.vqt_norm[bin + 1];
    assert!(v > 0.5, "expected strong normalized response at bin");
    assert!(v > vl && v > vr, "expected local max at target bin");
}

#[test]
fn lua_vqt_reads_bin() {
    let cap = 8192;
    let sr = 44_100;
    let vqt = Arc::new(RwLock::new(VQTState::new(sr, cap)));
    let bin = 36usize;
    let f0 = vqt_center_freq(bin);
    let samples = gen_sine(f0, sr, 10_000);
    {
        let mut w = vqt.write();
        for s in samples {
            w.ingest(s);
        }
        w.update();
    }
    set_global_vqt(vqt);
    let script = format!(
        "\
        function BOOT() cls(0) end\n\
        function TIC()\n\
            local v = vqt({bin})\n\
            if v > 0.5 then pix(0,0,9) end\n\
        end\n",
        bin = bin
    );
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    let runner = LuaRunner::new(fb.clone(), mem, &script).expect("lua init");
    runner.tick();
    assert_eq!(fb.borrow_mut().pix(0, 0, None), Some(9));
}

#[test]
fn vqt_whitened_arrays_are_finite() {
    let cap = 8192;
    let sr = 44_100;
    let mut vqt = VQTState::new(sr, cap);
    // Multi-tone to simulate broadband-ish content
    for i in 0..20_000 {
        let t = i as f32 / (sr as f32);
        let s = (2.0 * std::f32::consts::PI * 220.0 * t).sin()
            + (2.0 * std::f32::consts::PI * 880.0 * t).sin()
            + (2.0 * std::f32::consts::PI * 1760.0 * t).sin();
        vqt.ingest(s * 0.33);
    }
    vqt.update();
    for i in 0..vqt.bins_count() {
        assert!(vqt.vqt_w_raw[i].is_finite());
        assert!(vqt.vqt_w_sm[i].is_finite());
        assert!(vqt.vqt_w_norm[i].is_finite());
    }
}
