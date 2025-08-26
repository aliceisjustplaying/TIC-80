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

use tic80_rust::gfx::framebuffer::{dimensions, Framebuffer};
use tic80_rust::script::lua_runner::LuaRunner;
use tic80_rust::core::memory::Memory;

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
    // Program selection: first CLI arg as .lua script, else embedded default
    let args: Vec<String> = std::env::args().skip(1).collect();
    let script = if let Some(first) = args.first() {
        if first.ends_with(".lua") && Path::new(first).is_file() {
            match fs::read_to_string(first) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read {}: {}. Falling back to default cart.", first, e);
                    DEFAULT_LUA.to_string()
                }
            }
        } else {
            DEFAULT_LUA.to_string()
        }
    } else {
        DEFAULT_LUA.to_string()
    };
    let lua_runner = LuaRunner::new(fb.clone(), mem.clone(), &script).ok();

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
