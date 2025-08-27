use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use parking_lot::RwLock;
use std::sync::Arc;
use tic80_rust::audio::capture as audio_cap;
use tic80_rust::audio::fft::{set_global_fft, FFTState};
use tic80_rust::core::memory::Memory;
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

fn run() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    const SCALE: f64 = 3.0; // default integer scaling
    let (width, height) = dimensions();
    let size = LogicalSize::new((width as f64) * SCALE, (height as f64) * SCALE);
    let window = WindowBuilder::new()
        .with_title("rustic")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap();

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(width, height, surface_texture)?;

    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    let mut ticker = Ticker::new();
    // CLI parsing (minimal): flags + optional .lua path
    let mut args_iter = std::env::args().skip(1);
    let mut script_path: Option<String> = None;
    let mut list_audio = false;
    let mut audio_disable = false;
    let mut audio_device: Option<String> = None;
    let mut audio_vu = false;
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--list-audio" => list_audio = true,
            "--audio-disable" => audio_disable = true,
            "--audio-vu" => audio_vu = true,
            "--audio-device" => {
                if let Some(val) = args_iter.next() {
                    audio_device = Some(val);
                }
            }
            other => {
                if other.ends_with(".lua") && Path::new(other).is_file() {
                    script_path = Some(other.to_string());
                }
            }
        }
    }
    if list_audio {
        let list = audio_cap::list_input_devices();
        if list.is_empty() {
            println!("No input devices found.");
        } else {
            println!("Input devices:");
            for (i, name) in list.iter().enumerate() {
                println!("  {}: {}", i, name);
            }
        }
        return Ok(());
    }
    let script = if let Some(path) = script_path.as_ref() {
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Failed to read {}: {}. Falling back to default cart.",
                    path, e
                );
                DEFAULT_LUA.to_string()
            }
        }
    } else {
        DEFAULT_LUA.to_string()
    };
    let lua_runner = LuaRunner::new(fb.clone(), mem.clone(), &script).ok();

    // Optional audio capture
    struct AudioState {
        _handle: audio_cap::AudioCaptureHandle,
        cons: rtrb::Consumer<f32>,
        vu_enabled: bool,
        last_print: Instant,
        peak_acc: f32,
        fft: Arc<RwLock<FFTState>>,
        debug_fft: bool,
    }
    let mut audio_state: Option<AudioState> = None;
    // Optional debug flag for FFT bins
    let debug_fft = std::env::args().any(|a| a == "--debug-fft");

    if !audio_disable {
        let cap_cfg = audio_cap::AudioCaptureConfig {
            device_substr: audio_device.clone(),
            sample_rate: Some(44_100),
            ring_capacity: audio_cap::default_ring_capacity(),
        };
        match audio_cap::start_capture(cap_cfg) {
            Ok((handle, cons)) => {
                println!(
                    "Audio capture: '{}' @ {} Hz, {} ch",
                    handle.info.device_name, handle.info.sample_rate, handle.info.channels
                );
                if audio_vu {
                    println!("Audio VU: enabled (prints every ~1s)");
                }
                let fft_arc: Arc<RwLock<FFTState>> = Arc::new(RwLock::new(FFTState::new(
                    audio_cap::default_ring_capacity(),
                )));
                set_global_fft(fft_arc.clone());
                audio_state = Some(AudioState {
                    _handle: handle,
                    cons,
                    vu_enabled: audio_vu,
                    last_print: Instant::now(),
                    peak_acc: 0.0,
                    fft: fft_arc,
                    debug_fft,
                });
            }
            Err(e) => {
                eprintln!(
                    "Audio capture disabled ({}). Use --audio-disable to silence this.",
                    e
                );
            }
        }
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    let _ = pixels.resize_surface(size.width, size.height);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                if ticker.should_tick() {
                    if let Some(r) = &lua_runner {
                        r.tick();
                    }
                    // Simple VU meter from audio ring
                    if let Some(a) = audio_state.as_mut() {
                        // Drain available samples, feed analyzer, track peak
                        while let Ok(s) = a.cons.pop() {
                            a.peak_acc = a.peak_acc.max(s.abs());
                            if let Some(mut w) = a.fft.try_write() {
                                w.ingest(s);
                            }
                        }
                        if let Some(mut w) = a.fft.try_write() {
                            w.update();
                        }
                        if a.debug_fft {
                            // Print a small subset of normalized bins
                            let bins = {
                                let r = a.fft.read();
                                r.bins().min(16)
                            };
                            let mut line = String::from("FFT[0..16]: ");
                            if let Some(r) = a.fft.try_read() {
                                for i in 0..bins {
                                    line.push_str(&format!("{:.2} ", r.fft_sm[i]));
                                }
                            }
                            println!("{}", line);
                        }
                        if a.vu_enabled && a.last_print.elapsed() >= Duration::from_millis(1000) {
                            let peak = a.peak_acc.max(1e-9);
                            let db = 20.0 * peak.log10();
                            println!("VU: peak {:.3} ({:.1} dBFS)", peak, db);
                            a.peak_acc = 0.0;
                            a.last_print = Instant::now();
                        }
                    }
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();
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
