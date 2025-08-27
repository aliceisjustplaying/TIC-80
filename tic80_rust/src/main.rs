#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    rust_2018_idioms
)]
#![allow(
    clippy::many_single_char_names,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::multiple_crate_versions,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::cast_lossless,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::option_if_let_else,
    clippy::uninlined_format_args,
    clippy::significant_drop_tightening,
    clippy::match_same_arms,
    clippy::redundant_clone,
    clippy::struct_excessive_bools
)]

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use parking_lot::RwLock;
use std::sync::Arc;
use tic80_rust::audio::capture as audio_cap;
use tic80_rust::audio::fft::{set_global_fft, FFTState};
use tic80_rust::core::memory::Memory;
use tic80_rust::editor::code::{Area as CodeArea, CodeBuffer};
use tic80_rust::editor::ui::EditorUi;
use tic80_rust::gfx::framebuffer::{dimensions, Framebuffer};
use tic80_rust::script::lua_runner::LuaRunner;

// Simple fixed-step ticker at ~60 FPS
struct Ticker {
    last: Instant,
    step: Duration,
}

impl Ticker {
    fn new() -> Self {
        Self {
            last: Instant::now(),
            step: Duration::from_micros(16_667),
        }
    }
    fn should_tick(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last) >= self.step {
            self.last = now;
            true
        } else {
            false
        }
    }
}

const DEFAULT_LUA: &str = include_str!("../assets/default.lua");

// CLI args
struct Args {
    script_path: Option<PathBuf>,
    list_audio: bool,
    audio_disable: bool,
    audio_device: Option<String>,
    audio_vu: bool,
    debug_fft: bool,
    debug_fx: bool,
    quiet: bool,
    help: bool,
    editor: bool,
}

fn parse_args() -> Args {
    let mut args_iter = std::env::args().skip(1);
    let mut out = Args {
        script_path: None,
        list_audio: false,
        audio_disable: false,
        audio_device: None,
        audio_vu: false,
        debug_fft: false,
        debug_fx: false,
        quiet: false,
        help: false,
        editor: false,
    };
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-h" | "--help" => out.help = true,
            "--list-audio" => out.list_audio = true,
            "--audio-disable" => out.audio_disable = true,
            "--audio-vu" => out.audio_vu = true,
            "--debug-fft" => out.debug_fft = true,
            "--debug-fx" => out.debug_fx = true,
            "--quiet" => out.quiet = true,
            "--editor" => out.editor = true,
            "--audio-device" => {
                if let Some(val) = args_iter.next() {
                    out.audio_device = Some(val);
                }
            }
            other => {
                if other.ends_with(".lua") && Path::new(other).is_file() {
                    out.script_path = Some(PathBuf::from(other));
                }
            }
        }
    }
    out
}

fn create_window_and_pixels(
    event_loop: &EventLoop<()>,
    scale: f64,
) -> Result<(Window, Pixels), Error> {
    let (width, height) = dimensions();
    let size = LogicalSize::new((width as f64) * scale, (height as f64) * scale);
    let window = WindowBuilder::new()
        .with_title("rustic")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(event_loop)
        .unwrap();
    let ws = window.inner_size();
    let surface_texture = SurfaceTexture::new(ws.width, ws.height, &window);
    let pixels = Pixels::new(width, height, surface_texture)?;
    Ok((window, pixels))
}

// Optional audio capture
struct AudioState {
    handle: audio_cap::AudioCaptureHandle,
    cons: rtrb::Consumer<f32>,
    vu_enabled: bool,
    last_print: Instant,
    peak_acc: f32,
    fft: Arc<RwLock<FFTState>>,
    vqt: Arc<RwLock<tic80_rust::audio::vqt::VQTState>>,
    debug_fft: bool,
    last_fft_dbg: Instant,
    debug_fx: bool,
    fx_last: Instant,
    fx_fft_acc_ns: u128,
    fx_vqt_acc_ns: u128,
    fx_count: u64,
    last_pushed: u64,
    last_overflow: u64,
    underrun_count: u64,
    last_underrun: u64,
    consumed_total: u64,
    last_consumed: u64,
    ema_samples_per_tick: f64,
}

fn init_audio(args: &Args) -> Option<AudioState> {
    if args.audio_disable {
        return None;
    }
    let cap_cfg = audio_cap::AudioCaptureConfig {
        device_substr: args.audio_device.clone(),
        sample_rate: Some(44_100),
        ring_capacity: audio_cap::default_ring_capacity(),
    };
    match audio_cap::start_capture(cap_cfg) {
        Ok((handle, cons)) => {
            println!(
                "Audio capture: '{}' @ {} Hz, {} ch",
                handle.info.device_name, handle.info.sample_rate, handle.info.channels
            );
            if args.audio_vu {
                println!("Audio VU: enabled (prints every ~1s)");
            }
            let fft_arc: Arc<RwLock<FFTState>> = Arc::new(RwLock::new(FFTState::new(
                audio_cap::default_ring_capacity(),
            )));
            set_global_fft(fft_arc.clone());
            let vqt_arc: Arc<RwLock<tic80_rust::audio::vqt::VQTState>> =
                Arc::new(RwLock::new(tic80_rust::audio::vqt::VQTState::new(
                    handle.info.sample_rate,
                    audio_cap::default_ring_capacity(),
                )));
            tic80_rust::audio::vqt::set_global_vqt(vqt_arc.clone());
            Some(AudioState {
                handle,
                cons,
                vu_enabled: args.audio_vu,
                last_print: Instant::now(),
                peak_acc: 0.0,
                fft: fft_arc,
                vqt: vqt_arc,
                debug_fft: args.debug_fft,
                last_fft_dbg: Instant::now(),
                debug_fx: args.debug_fx,
                fx_last: Instant::now(),
                fx_fft_acc_ns: 0,
                fx_vqt_acc_ns: 0,
                fx_count: 0,
                last_pushed: 0,
                last_overflow: 0,
                underrun_count: 0,
                last_underrun: 0,
                consumed_total: 0,
                last_consumed: 0,
                ema_samples_per_tick: 0.0,
            })
        }
        Err(e) => {
            eprintln!(
                "Audio capture disabled ({}). Use --audio-disable to silence this.",
                e
            );
            None
        }
    }
}

fn print_devices_and_exit() {
    let list = audio_cap::list_input_devices();
    if list.is_empty() {
        println!("No input devices found.");
    } else {
        println!("Input devices:");
        for (i, name) in list.iter().enumerate() {
            println!("  {}: {}", i, name);
        }
    }
}

fn print_help() {
    let prog = std::env::args().next().map_or_else(
        || "tic80_rust".to_string(),
        |p| {
            std::path::Path::new(&p)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("tic80_rust")
                .to_string()
        },
    );
    println!(
        "Usage: {prog} [OPTIONS] [CART.lua]\n\nOptions:\n  -h, --help                 Show this help message and exit\n      --quiet                Suppress once-only warnings and Lua BOOT()/TIC() error prints\n      --list-audio           List input audio devices and exit\n      --audio-device <SUBSTR>  Select input device by substring match (case-insensitive)\n      --audio-disable        Disable audio capture and analysis\n      --audio-vu             Print VU peak dBFS once per second\n      --debug-fft            Print first 16 FFT bins (smoothed, normalized) ~every 500 ms\n      --debug-fx             Print per-second FX timings plus ring stats (dp/ovf/underrun/consumed, EMA samples/tick, occupancy)\n\nArguments:\n  CART.lua                   Optional path to a Lua cart; defaults to bundled demo when omitted\n\nNotes:\n- Window: fixed 240x136 internal resolution with integer scaling in a desktop window.\n- Audio: selects nearest supported sample rate to 44100 Hz and logs the choice.\n- Ring stats (with --debug-fx):\n    dp  = pushed samples since last report\n    ovf = overflows delta (and total) from producer\n    underrun = consumer had no data to read\n    consumed = samples pulled into analyzers (delta)\n    EMA samples/tick = moving average of consumed samples per game tick\n    occupancy = estimated ring fill vs. capacity\n\nExamples:\n  {prog}                                  # run bundled cart\n  {prog} assets/alt.lua                    # run a local cart\n  {prog} --list-audio                      # show devices and exit\n  {prog} --audio-device BlackHole --audio-vu\n  {prog} assets/fft_test.lua --debug-fft --debug-fx\n  {prog} assets/your_cart.lua --quiet\n        "
    );
}

fn load_script(script_path: Option<&PathBuf>) -> String {
    if let Some(path) = script_path {
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Failed to read {}: {}. Falling back to default cart.",
                    path.display(),
                    e
                );
                DEFAULT_LUA.to_string()
            }
        }
    } else {
        DEFAULT_LUA.to_string()
    }
}

fn run() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let (window, mut pixels) = create_window_and_pixels(&event_loop, 3.0)?; // default integer scaling
    let window_scale = 3.0f64;

    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    let mut ticker = Ticker::new();

    let args = parse_args();
    if args.help {
        print_help();
        return Ok(());
    }
    if args.help {
        print_help();
        return Ok(());
    }
    if args.list_audio {
        print_devices_and_exit();
        return Ok(());
    }
    tic80_rust::script::lua_runner::set_quiet(args.quiet);
    let (lua_runner, mut editor_ui, mut code_buf) = if args.editor {
        let initial = load_script(args.script_path.as_ref());
        (
            None,
            Some(EditorUi::new(window_scale)),
            Some(CodeBuffer::from_text(&initial)),
        )
    } else {
        let script = load_script(args.script_path.as_ref());
        let lr = match LuaRunner::new(fb.clone(), mem.clone(), &script) {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("Lua initialization error: {e}");
                None
            }
        };
        (lr, None, None)
    };
    let mut audio_state = init_audio(&args);
    let mut warned_no_tic = false;
    let mut last_cursor_fb: Option<(i32, i32)> = None;

    #[allow(clippy::cognitive_complexity)]
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => {
                    if let Some(ui) = editor_ui.as_ref() {
                        let (fx, fy) = ui.window_to_fb(position.x, position.y);
                        last_cursor_fb = Some((fx, fy));
                    }
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, .. } => {
                    if let (Some((fx, fy)), Some(ui)) = (last_cursor_fb, editor_ui.as_mut()) {
                        ui.on_click_fb(fx, fy);
                    }
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let (Some(ui), Some(cb)) = (editor_ui.as_ref(), code_buf.as_mut()) {
                        if ui.active == tic80_rust::editor::ui::Tab::Code
                            && input.state == ElementState::Pressed
                        {
                            if let Some(key) = input.virtual_keycode {
                                match key {
                                    VirtualKeyCode::Left => cb.move_left(),
                                    VirtualKeyCode::Right => cb.move_right(),
                                    VirtualKeyCode::Up => cb.move_up(),
                                    VirtualKeyCode::Down => cb.move_down(),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                WindowEvent::Resized(size) => {
                    let _ = pixels.resize_surface(size.width, size.height);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                if ticker.should_tick() {
                    if editor_ui.is_none() {
                        if let Some(r) = &lua_runner {
                            r.tick();
                        } else if !warned_no_tic && !args.quiet {
                            eprintln!("No TIC() to run; idle");
                            warned_no_tic = true;
                        }
                    }
                    // Simple VU meter from audio ring
                    if let Some(a) = audio_state.as_mut() {
                        // Drain available samples, feed analyzer with a single write lock, track peak
                        {
                            let t0 = Instant::now();
                            let mut w = a.fft.write();
                            if a.cons.is_empty() {
                                a.underrun_count += 1;
                            }
                            let mut consumed_this_tick: u64 = 0;
                            while let Ok(s) = a.cons.pop() {
                                a.peak_acc = a.peak_acc.max(s.abs());
                                w.ingest(s);
                                // Also feed VQT buffer
                                a.vqt.write().ingest(s);
                                consumed_this_tick += 1;
                            }
                            a.consumed_total = a.consumed_total.saturating_add(consumed_this_tick);
                            // EMA of samples/tick (smoothing factor 0.9)
                            let alpha = 0.9f64;
                            a.ema_samples_per_tick = a
                                .ema_samples_per_tick
                                .mul_add(alpha, (consumed_this_tick as f64) * (1.0 - alpha));
                            w.update();
                            let t1 = Instant::now();
                            a.vqt.write().update();
                            let t2 = Instant::now();
                            if a.debug_fx {
                                a.fx_fft_acc_ns += (t1 - t0).as_nanos();
                                a.fx_vqt_acc_ns += (t2 - t1).as_nanos();
                                a.fx_count += 1;
                                if a.fx_last.elapsed() >= Duration::from_millis(1000) {
                                    let n = a.fx_count.max(1);
                                    #[allow(clippy::cast_precision_loss)]
                                    let avg_fft_ms = (a.fx_fft_acc_ns as f64) / 1.0e6 / (n as f64);
                                    #[allow(clippy::cast_precision_loss)]
                                    let avg_vqt_ms = (a.fx_vqt_acc_ns as f64) / 1.0e6 / (n as f64);
                                    // ring stats (if audio enabled)
                                    if let Some(h) = Some(&a.handle) {
                                        let pushed = h
                                            .pushed_count
                                            .load(std::sync::atomic::Ordering::Relaxed);
                                        let ovf = h
                                            .overflow_count
                                            .load(std::sync::atomic::Ordering::Relaxed);
                                        let dp = pushed.saturating_sub(a.last_pushed);
                                        let dofv = ovf.saturating_sub(a.last_overflow);
                                        let du = a.underrun_count.saturating_sub(a.last_underrun);
                                        let dc = a.consumed_total.saturating_sub(a.last_consumed);
                                        let occ_est = pushed
                                            .saturating_sub(ovf)
                                            .saturating_sub(a.consumed_total);
                                        #[allow(clippy::cast_possible_truncation)]
                                        let occ = occ_est.min(h.ring_capacity as u64) as usize;
                                        let occ_pct = (occ as f64) / (h.ring_capacity as f64) * 100.0;
                                        println!(
                                            "FX avg ms: fft={avg_fft_ms:.3}, vqt={avg_vqt_ms:.3} ({n} ticks), dp={dp}, ovf+={dofv} (tot {ovf}), underrun+={du} (tot {uc}), cons+={dc}, ema_spt={ema:.1}, occ={occ}/{cap} ({occ_pct:.0}%)",
                                            uc = a.underrun_count,
                                            ema = a.ema_samples_per_tick,
                                            cap = h.ring_capacity,
                                            occ_pct = occ_pct,
                                            occ = occ
                                        );
                                        a.last_pushed = pushed;
                                        a.last_overflow = ovf;
                                        a.last_underrun = a.underrun_count;
                                        a.last_consumed = a.consumed_total;
                                    } else {
                                        println!(
                                            "FX avg ms: fft={avg_fft_ms:.3}, vqt={avg_vqt_ms:.3} ({n} ticks)"
                                        );
                                    }
                                    a.fx_fft_acc_ns = 0;
                                    a.fx_vqt_acc_ns = 0;
                                    a.fx_count = 0;
                                    a.fx_last = Instant::now();
                                }
                            }
                        }
                        if a.debug_fft && a.last_fft_dbg.elapsed() >= Duration::from_millis(500) {
                            // Print a small subset of normalized bins
                            let bins = {
                                let r = a.fft.read();
                                r.bins().min(16)
                            };
                            let mut line = String::from("FFT[0..16]: ");
                            {
                                let r = a.fft.read();
                                for i in 0..bins {
                                    use std::fmt::Write as _;
                                    let _ = write!(&mut line, "{:.2} ", r.fft_sm[i]);
                                }
                            }
                            println!("{line}");
                            a.last_fft_dbg = Instant::now();
                        }
                        if a.vu_enabled && a.last_print.elapsed() >= Duration::from_millis(1000) {
                            let peak = a.peak_acc.max(1e-9);
                            let db = 20.0 * peak.log10();
                            println!("VU: peak {peak:.3} ({db:.1} dBFS)");
                            a.peak_acc = 0.0;
                            a.last_print = Instant::now();
                        }
                    }
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();
                if let Some(ui) = editor_ui.as_ref() {
                    let mut fbb = fb.borrow_mut();
                    ui.draw(&mut fbb);
                    if ui.active == tic80_rust::editor::ui::Tab::Code {
                        if let Some(cb) = code_buf.as_mut() {
                            let area = CodeArea { x: 0, y: 12, w: 240, h: 124 };
                            cb.draw(&mut fbb, area);
                        }
                    }
                }
                fb.borrow().blit_to_rgba(frame);
                if let Err(err) = pixels.render() {
                    eprintln!("Render error: {err}");
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Application error: {err}");
    }
}
