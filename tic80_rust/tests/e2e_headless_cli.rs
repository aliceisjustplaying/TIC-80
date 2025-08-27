use std::path::PathBuf;
use std::process::Command;

use png::Decoder;

fn run_and_capture(args: &[&str], out_path: &PathBuf) {
    let bin = env!("CARGO_BIN_EXE_tic80_rust");
    let status = Command::new(bin)
        .args(args)
        .status()
        .expect("failed to spawn binary");
    assert!(status.success(), "runner exited with failure: {status:?}");

    // Decode and verify dimensions > 0
    let file = std::fs::File::open(out_path).expect("screenshot file not created");
    let mut reader = Decoder::new(file).read_info().expect("decode header");
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("read frame");
    assert!(info.width > 0 && info.height > 0);
}

#[test]
fn e2e_headless_default_cart_screenshot() {
    let mut p = std::env::temp_dir();
    p.push(format!("tic80_e2e_default_{}.png", std::process::id()));
    let args = [
        "--headless",
        "--screenshot",
        p.to_str().unwrap(),
        "--screenshot-frame",
        "1",
    ];
    run_and_capture(&args, &p);
    let _ = std::fs::remove_file(&p);
}

#[test]
fn e2e_headless_editor_screenshot() {
    let mut p = std::env::temp_dir();
    p.push(format!("tic80_e2e_editor_{}.png", std::process::id()));
    let args = [
        "--headless",
        "--editor",
        "--screenshot",
        p.to_str().unwrap(),
    ];
    run_and_capture(&args, &p);
    let _ = std::fs::remove_file(&p);
}
