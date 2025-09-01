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

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{
    ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent,
};
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
    // Editor helpers (headless diagnostics)
    editor_demo_select: bool,
    // Screenshot/headless
    headless: bool,
    screenshot_path: Option<PathBuf>,
    screenshot_scale: u32,
    screenshot_frame: Option<u32>,
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
        editor_demo_select: false,
        headless: false,
        screenshot_path: None,
        screenshot_scale: 1,
        screenshot_frame: None,
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
            "--editor-demo-select" => out.editor_demo_select = true,
            "--headless" => out.headless = true,
            "--screenshot" => {
                if let Some(val) = args_iter.next() {
                    out.screenshot_path = Some(PathBuf::from(val));
                }
            }
            "--screenshot-scale" => {
                if let Some(val) = args_iter.next() {
                    if let Ok(n) = val.parse::<u32>() {
                        out.screenshot_scale = n.max(1);
                    }
                }
            }
            "--screenshot-frame" => {
                if let Some(val) = args_iter.next() {
                    if let Ok(n) = val.parse::<u32>() {
                        out.screenshot_frame = Some(n);
                    }
                }
            }
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
) -> anyhow::Result<(Window, Pixels)> {
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
        "Usage: {prog} [OPTIONS] [CART.lua]\n\nOptions:\n  -h, --help                 Show this help message and exit\n      --quiet                Suppress once-only warnings and Lua BOOT()/TIC() error prints\n      --editor               Launch the editor UI (CODE/CONSOLE)\n      --editor-demo-select   In headless editor mode, create a 3-line demo selection before drawing\n      --headless             Run offscreen without opening a window (for screenshots/CI)\n      --screenshot <PATH>    Save a screenshot and exit (first frame by default)\n      --screenshot-scale <N> Integer scale for screenshot (default 1)\n      --screenshot-frame <N> Capture after N frames (windowed/headless)\n      --list-audio           List input audio devices and exit\n      --audio-device <SUBSTR>  Select input device by substring match (case-insensitive)\n      --audio-disable        Disable audio capture and analysis\n      --audio-vu             Print VU peak dBFS once per second\n      --debug-fft            Print first 16 FFT bins (smoothed, normalized) ~every 500 ms\n      --debug-fx             Print per-second FX timings plus ring stats (dp/ovf/underrun/consumed, EMA samples/tick, occupancy)\n\nArguments:\n  CART.lua                   Optional path to a Lua cart; defaults to bundled demo when omitted\n\nNotes:\n- Window: fixed 240x136 internal resolution with integer scaling in a desktop window.\n- Audio: selects nearest supported sample rate to 44100 Hz and logs the choice.\n- Ring stats (with --debug-fx): dp=pushed, ovf=overflows, underrun=no-data, consumed=samples, EMA samples/tick, occupancy.\n\nExamples:\n  {prog} --screenshot out.png --screenshot-scale 3\n  {prog} --headless --screenshot out.png --screenshot-frame 60\n  {prog} --editor\n        ");
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

fn run() -> anyhow::Result<()> {
    // Parse early to allow headless path
    let args = parse_args();
    if args.help {
        print_help();
        return Ok(());
    }
    if args.headless {
        return run_headless(&args);
    }

    let event_loop = EventLoop::new();
    let (window, mut pixels) = create_window_and_pixels(&event_loop, 3.0)?; // default integer scaling
    let window_scale = 3.0f64;

    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    let mut ticker = Ticker::new();
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
    let mut frame_counter: u32 = 0;
    let mut one_off_saved = false;
    let mut modifiers = ModifiersState::empty();

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
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::F12),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    // Save a timestamped screenshot under ./screenshots
                    let (w, h) = dimensions();
                    let mut rgba = vec![0u8; (w * h * 4) as usize];
                    fb.borrow().blit_to_rgba(&mut rgba);
                    if let Err(e) = save_timestamped_screenshot(&args, &rgba, w, h) {
                        eprintln!("Screenshot error: {e}");
                    }
                }
                WindowEvent::ReceivedCharacter(ch) => {
                    if let (Some(ui), Some(cb)) = (editor_ui.as_ref(), code_buf.as_mut()) {
                        if ui.active == tic80_rust::editor::ui::Tab::Code {
                            // Ignore character input when Cmd/Ctrl is held (shortcut) to avoid inserting letters
                            if modifiers.ctrl() || modifiers.logo() {
                                return;
                            }
                            if ch == '\n' {
                                cb.insert_newline();
                            } else if ch == '\t' {
                                // handled in KeyboardInput for block indent/outdent
                            } else if !ch.is_control() {
                                cb.insert_char(ch);
                            }
                        }
                    }
                }
                WindowEvent::ModifiersChanged(m) => {
                    modifiers = m;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let (Some(ui), Some(cb)) = (editor_ui.as_ref(), code_buf.as_mut()) {
                        if ui.active == tic80_rust::editor::ui::Tab::Code
                            && input.state == ElementState::Pressed
                        {
                            if let Some(key) = input.virtual_keycode {
                                // Prefer latest modifiers from event loop; fall back to key-based detection
                                #[allow(deprecated)]
                                let m = input.modifiers;
                                let shift = m.shift();
                                let ctrl = m.ctrl();
                                let cmd = m.logo();
                                // Shortcuts (cmd/ctrl)
                                if ctrl || cmd {
                                    match key {
                                        VirtualKeyCode::C => {
                                            if let Some(text) = cb.copy_selection_text() {
                                                let _ = set_clipboard_text(&text);
                                            }
                                            return;
                                        }
                                        VirtualKeyCode::X => {
                                            if let Some(text) = cb.cut_selection_text() {
                                                let _ = set_clipboard_text(&text);
                                            }
                                            return;
                                        }
                                        VirtualKeyCode::V => {
                                            if let Ok(text) = get_clipboard_text() {
                                                cb.paste_text(&text);
                                            }
                                            return;
                                        }
                                        VirtualKeyCode::Z => {
                                            if shift { cb.redo(); } else { cb.undo(); }
                                            return;
                                        }
                                        VirtualKeyCode::Y => { cb.redo(); return; }
                                        VirtualKeyCode::A => { cb.select_all(); return; }
                                        _ => {}
                                    }
                                }
                                // Navigation and editing
                                match key {
                                    VirtualKeyCode::PageUp => { if shift { cb.ensure_selection_anchor(); } else { cb.clear_selection(); } cb.page_up(18); },
                                    VirtualKeyCode::PageDown => { if shift { cb.ensure_selection_anchor(); } else { cb.clear_selection(); } cb.page_down(18); },
                                    VirtualKeyCode::Home if ctrl || cmd => { cb.doc_home(); },
                                    VirtualKeyCode::End if ctrl || cmd => { cb.doc_end(); },
                                    VirtualKeyCode::Left => { if shift { cb.ensure_selection_anchor(); } else { cb.clear_selection(); } cb.move_left(); },
                                    VirtualKeyCode::Right => { if shift { cb.ensure_selection_anchor(); } else { cb.clear_selection(); } cb.move_right(); },
                                    VirtualKeyCode::Up => { if shift { cb.ensure_selection_anchor(); } else { cb.clear_selection(); } cb.move_up(); },
                                    VirtualKeyCode::Down => { if shift { cb.ensure_selection_anchor(); } else { cb.clear_selection(); } cb.move_down(); },
                                    VirtualKeyCode::Back => { cb.backspace(); },
                                    VirtualKeyCode::Delete => { cb.delete_forward(); },
                                    VirtualKeyCode::Return => { cb.insert_newline(); },
                                    VirtualKeyCode::Tab => {
                                        if shift { cb.block_outdent(); } else if cb.has_selection() { cb.block_indent(); } else { cb.insert_tab(); }
                                    },
                                    VirtualKeyCode::Home => { if shift { cb.ensure_selection_anchor(); } else { cb.clear_selection(); } cb.home(); },
                                    VirtualKeyCode::End => { if shift { cb.ensure_selection_anchor(); } else { cb.clear_selection(); } cb.end(); },
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
                            let area = CodeArea { x: 0, y: 7, w: 240, h: 129 };
                            cb.draw(&mut fbb, area);
                        }
                    }
                }
                fb.borrow().blit_to_rgba(frame);
                // One-off screenshot capture if requested
                if !one_off_saved {
                    let target = args.screenshot_frame.unwrap_or(0);
                    if args.screenshot_path.is_some() && frame_counter >= target {
                        if let Err(e) = save_screenshot_from_rgba(&args, frame) { eprintln!("Screenshot error: {e}"); }
                        one_off_saved = true;
                        *control_flow = ControlFlow::Exit;
                    }
                }
                if let Err(err) = pixels.render() {
                    eprintln!("Render error: {err}");
                    *control_flow = ControlFlow::Exit;
                }
                frame_counter = frame_counter.saturating_add(1);
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

// -- Screenshot helpers and headless path -------------------------------------------------------

fn save_screenshot_from_rgba(args: &Args, rgba: &[u8]) -> anyhow::Result<()> {
    use tic80_rust::util::image::{save_png_rgba, scale_rgba_nn};
    let path = args
        .screenshot_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("screenshot path not provided"))?;
    let (w, h) = dimensions();
    let scaled = if args.screenshot_scale <= 1 {
        rgba.to_vec()
    } else {
        scale_rgba_nn(rgba, w, h, args.screenshot_scale)
    };
    let sw = w * args.screenshot_scale;
    let sh = h * args.screenshot_scale;
    save_png_rgba(path, sw, sh, &scaled)?;
    println!(
        "Saved screenshot: {} ({}x{}, scale {})",
        path.display(),
        sw,
        sh,
        args.screenshot_scale
    );
    Ok(())
}

fn save_timestamped_screenshot(args: &Args, rgba: &[u8], w: u32, h: u32) -> anyhow::Result<()> {
    use tic80_rust::util::image::{save_png_rgba, scale_rgba_nn};
    let dir = PathBuf::from("screenshots");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    let now = chrono::Local::now();
    let fname = format!("scr-{}.png", now.format("%Y%m%d-%H%M%S"));
    let path = dir.join(fname);
    let scale = args.screenshot_scale.max(1);
    let scaled = if scale <= 1 {
        rgba.to_vec()
    } else {
        scale_rgba_nn(rgba, w, h, scale)
    };
    let sw = w * scale;
    let sh = h * scale;
    save_png_rgba(&path, sw, sh, &scaled)?;
    println!(
        "Saved screenshot: {} ({}x{}, scale {})",
        path.display(),
        sw,
        sh,
        scale
    );
    Ok(())
}

fn run_headless(args: &Args) -> anyhow::Result<()> {
    // Prepare framebuffer + memory
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    // Render either editor UI or cart ticks
    if args.editor {
        let initial = load_script(args.script_path.as_ref());
        let ui = EditorUi::new(1.0);
        let mut code = CodeBuffer::from_text(&initial);
        if args.editor_demo_select {
            apply_demo_selection(&mut code);
        }
        {
            let mut fbb = fb.borrow_mut();
            ui.draw(&mut fbb);
            let area = CodeArea { x: 0, y: 7, w: 240, h: 129 };
            code.draw(&mut fbb, area);
        }
    } else {
        let script = load_script(args.script_path.as_ref());
        let lr = LuaRunner::new(fb.clone(), mem.clone(), &script)?;
        // Match windowed timing: capture on redraw number target+1
        let ticks = args.screenshot_frame.unwrap_or(0).saturating_add(1);
        for _ in 0..ticks {
            lr.tick();
        }
    }
    // Save from framebuffer
    let (w, h) = dimensions();
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    fb.borrow().blit_to_rgba(&mut rgba);
    save_screenshot_from_rgba(args, &rgba)?;
    Ok(())
}

// Create a 3-line demo selection with ragged edges to inspect drop shadows.
fn apply_demo_selection(cb: &mut CodeBuffer) {
    // Choose lines 6..=8 (0-based) which are likely to exist in default carts.
    let l0 = 6usize;
    let l2 = 8usize;
    // Start mid-line, end a few chars into line 8 to create overhangs.
    let c0 = (cb.line_len(l0) / 2).max(2);
    let c2 = cb.line_len(l2).min(8);
    cb.caret_line = l0;
    cb.caret_col = c0;
    cb.start_selection();
    cb.caret_line = l2;
    cb.caret_col = c2;
}

// -- Clipboard helpers -------------------------------------------------------------------------

fn set_clipboard_text(text: &str) -> anyhow::Result<()> {
    let mut cb = arboard::Clipboard::new()?;
    cb.set_text(text.to_string())?;
    Ok(())
}

fn get_clipboard_text() -> anyhow::Result<String> {
    let mut cb = arboard::Clipboard::new()?;
    let t = cb.get_text()?;
    Ok(t)
}
