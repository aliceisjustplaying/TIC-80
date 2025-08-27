use std::path::PathBuf;

use png::Decoder;
use tic80_rust::gfx::framebuffer::{dimensions, Framebuffer};
use tic80_rust::util::image::{encode_png_rgba_to_vec, save_png_rgba, scale_rgba_nn};

#[test]
fn save_scaled_png_has_expected_dimensions() {
    let (w, h) = dimensions();
    let mut fb = Framebuffer::new();
    // Draw something deterministic
    fb.cls(1);
    fb.rect(10, 10, 50, 30, 12);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    fb.blit_to_rgba(&mut rgba);

    // 1) In-memory roundtrip and decode
    let bytes = encode_png_rgba_to_vec(w, h, &rgba).expect("encode");
    let mut reader = Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .expect("decoder");
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("frame");
    assert_eq!(info.width, w);
    assert_eq!(info.height, h);

    // 2) Scaled save to a temp file and decode
    let scale = 3;
    let scaled = scale_rgba_nn(&rgba, w, h, scale);
    let sw = w * scale;
    let sh = h * scale;
    let mut p = std::env::temp_dir();
    p.push(format!("tic80_screenshot_test_{}.png", std::process::id()));
    save_png_rgba(&PathBuf::from(&p), sw, sh, &scaled).expect("save");
    let mut reader = Decoder::new(std::fs::File::open(&p).expect("open"))
        .read_info()
        .unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!(info.width, sw);
    assert_eq!(info.height, sh);
    // Cleanup best-effort
    let _ = std::fs::remove_file(&p);
}
