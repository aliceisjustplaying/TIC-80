use std::rc::Rc;
use std::time::{Duration, Instant};
use std::cell::RefCell;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use mlua::{Lua, Function, RegistryKey, Result as LuaResult};

const WIDTH: u32 = 240;
const HEIGHT: u32 = 136;

// Default 16-color TIC-80 palette (sRGB) as RGBA8
const PALETTE: [(u8, u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00, 0xFF), (0x1D, 0x2B, 0x53, 0xFF), (0x7E, 0x25, 0x53, 0xFF), (0x00, 0x87, 0x51, 0xFF),
    (0xAB, 0x52, 0x36, 0xFF), (0x5F, 0x57, 0x4F, 0xFF), (0xC2, 0xC3, 0xC7, 0xFF), (0xFF, 0xF1, 0xE8, 0xFF),
    (0xFF, 0x00, 0x4D, 0xFF), (0xFF, 0xA3, 0x00, 0xFF), (0xFF, 0xEC, 0x27, 0xFF), (0x00, 0xE4, 0x36, 0xFF),
    (0x29, 0xAD, 0xFF, 0xFF), (0x83, 0x76, 0x9C, 0xFF), (0xFF, 0x77, 0xA8, 0xFF), (0xFF, 0xCC, 0xAA, 0xFF),
];

struct Framebuffer {
    // 240x136 palette indices (0..15)
    idx: Vec<u8>,
}

impl Framebuffer {
    fn new() -> Self {
        Self { idx: vec![0; (WIDTH * HEIGHT) as usize] }
    }

    // cls(color): fill framebuffer with palette index
    fn cls(&mut self, color: u8) {
        self.idx.fill(color & 0x0F);
    }

    // pix(x,y[,color]): if Some(color) -> write; else -> read
    fn pix(&mut self, x: i32, y: i32, color: Option<u8>) -> Option<u8> {
        if x < 0 || y < 0 || x as u32 >= WIDTH || y as u32 >= HEIGHT {
            return None;
        }
        let i = (y as u32 * WIDTH + x as u32) as usize;
        match color {
            Some(c) => {
                self.idx[i] = c & 0x0F;
                None
            }
            None => Some(self.idx[i] & 0x0F),
        }
    }

    // Blit to RGBA buffer for pixels
    fn blit_to_rgba(&self, rgba: &mut [u8]) {
        for (i, idx) in self.idx.iter().copied().enumerate() {
            let (r, g, b, a) = PALETTE[(idx & 0x0F) as usize];
            let o = i * 4;
            rgba[o] = r;
            rgba[o + 1] = g;
            rgba[o + 2] = b;
            rgba[o + 3] = a;
        }
    }
}

// Simple fixed-step ticker at ~60 FPS
struct Ticker {
    last: Instant,
    step: Duration,
}

impl Ticker {
    fn new() -> Self {
        Self { last: Instant::now(), step: Duration::from_micros(16_667) }
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

const DEFAULT_LUA: &str = r#"
-- Minimal demo using cls and pix
local t = 0
function BOOT()
  cls(0)
end
function TIC()
  if t % 30 == 0 then cls(((t // 30) % 16)) end
  local cx, cy = 120, 68
  for dx = -10, 10 do pix(cx + dx, cy, 15) end
  for dy = -10, 10 do pix(cx, cy + dy, 15) end
  t = t + 1
end
"#;

struct LuaRunner {
    lua: Lua,
    tic_key: Option<RegistryKey>,
}

impl LuaRunner {
    fn new(fb: Rc<RefCell<Framebuffer>>, script_src: &str) -> LuaResult<Self> {
        let lua = Lua::new();
        let tic_key = {
            let globals = lua.globals();

            // Bind cls(color)
            let fb_cls = fb.clone();
            let cls_fn = lua.create_function(move |_, color: Option<u8>| {
                fb_cls.borrow_mut().cls(color.unwrap_or(0));
                Ok(())
            })?;
            globals.set("cls", cls_fn)?;

            // Bind pix(x,y[,color]) -> color or nil
            let fb_pix = fb.clone();
            let pix_fn = lua.create_function(move |_, (x, y, color): (i32, i32, Option<u8>)| {
                let res = fb_pix.borrow_mut().pix(x, y, color);
                Ok(res)
            })?;
            globals.set("pix", pix_fn)?;

            // Load script
            lua.load(script_src).set_name("cart").exec()?;

            // Call BOOT() if present
            if let Ok(boot) = globals.get::<_, Function>("BOOT") {
                let _ = boot.call::<_, ()>(());
            }

            // Cache TIC if present
            match globals.get::<_, Option<Function>>("TIC")? {
                Some(f) => Some(lua.create_registry_value(f)?),
                None => None,
            }
        };

        Ok(Self { lua, tic_key })
    }

    fn tick(&self) {
        if let Some(key) = &self.tic_key {
            if let Ok(func) = self.lua.registry_value::<Function>(key) {
                let _ = func.call::<_, ()>(());
            }
        }
    }
}

fn run() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let scale = 3.0f64; // default integer scaling
    let size = LogicalSize::new((WIDTH as f64) * scale, (HEIGHT as f64) * scale);
    let window = WindowBuilder::new()
        .with_title("tic80_rust – Milestone 1 (GUI + cls/pix)")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap();

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut ticker = Ticker::new();

    // Lua runner with default script
    let lua_runner = LuaRunner::new(fb.clone(), DEFAULT_LUA).ok();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), state: ElementState::Pressed, .. }, .. } => {
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::Resized(size) => {
                    let _ = pixels.resize_surface(size.width, size.height);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                if ticker.should_tick() {
                    if let Some(r) = &lua_runner { r.tick(); }
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();
                fb.borrow().blit_to_rgba(frame);
                let _ = pixels.render();
            }
            _ => {}
        }
    });

    // Unreachable with current event loop
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Application error: {err}");
    }
}
