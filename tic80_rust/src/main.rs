use std::rc::Rc;
use std::time::{Duration, Instant};
use std::cell::RefCell;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use mlua::{Lua, Function, RegistryKey, Result as LuaResult, MultiValue, Value};
use std::sync::OnceLock;

const WIDTH: u32 = 240;
const HEIGHT: u32 = 136;

// Default 16-color TIC-80 palette (sRGB) as RGBA8
const PALETTE: [(u8, u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00, 0xFF), (0x1D, 0x2B, 0x53, 0xFF), (0x7E, 0x25, 0x53, 0xFF), (0x00, 0x87, 0x51, 0xFF),
    (0xAB, 0x52, 0x36, 0xFF), (0x5F, 0x57, 0x4F, 0xFF), (0xC2, 0xC3, 0xC7, 0xFF), (0xFF, 0xF1, 0xE8, 0xFF),
    (0xFF, 0x00, 0x4D, 0xFF), (0xFF, 0xA3, 0x00, 0xFF), (0xFF, 0xEC, 0x27, 0xFF), (0x00, 0xE4, 0x36, 0xFF),
    (0x29, 0xAD, 0xFF, 0xFF), (0x83, 0x76, 0x9C, 0xFF), (0xFF, 0x77, 0xA8, 0xFF), (0xFF, 0xCC, 0xAA, 0xFF),
];

// TIC-80 default 5x8 font as 8 rows per glyph, 5 bits per row (LSB masked with 0x1F), ASCII indexed.
static FONT_TEXT: &str = include_str!("../../src/core/font.inl");
static FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();

fn font_bytes() -> &'static [u8] {
    FONT_BYTES.get_or_init(|| {
        // Parse C-style hex list into bytes
        let mut out = Vec::with_capacity(1024);
        for tok in FONT_TEXT.split(|c: char| c.is_whitespace() || c == ',') {
            if tok.is_empty() { continue; }
            let t = tok.trim();
            let val = if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                u8::from_str_radix(hex, 16).ok()
            } else {
                // fallback: decimal
                t.parse::<u8>().ok()
            };
            if let Some(b) = val { out.push(b); }
        }
        out
    })
}

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

    // Draw a line using integer Bresenham
    fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u8) {
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let c = color & 0x0F;
        loop {
            let _ = self.pix(x0, y0, Some(c));
            if x0 == x1 && y0 == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x0 += sx; }
            if e2 <= dx { err += dx; y0 += sy; }
        }
    }

    // Filled rectangle
    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        if w <= 0 || h <= 0 { return; }
        let c = color & 0x0F;
        for yy in y..y+h {
            for xx in x..x+w {
                let _ = self.pix(xx, yy, Some(c));
            }
        }
    }

    // Print text using TIC-80 default font (5x8 glyphs, 1px spacing)
    fn print_text(&mut self, text: &str, x: i32, mut y: i32, color: u8, fixed: bool, scale: i32, _small: bool) -> i32 {
        // Match TIC-80 print/drawText:
        // - draw from 8x8 tile
        // - fixed: draw full 8 columns, advance by TIC_FONT_WIDTH (6)
        // - non-fixed: trim empty columns [start,end) and advance by (width+1) if width>0 else TIC_FONT_WIDTH
        const GLYPH_W: usize = 8;
        const GLYPH_H: usize = 8;
        const ADV: i32 = 6; // TIC_FONT_WIDTH
        if scale <= 0 { return 0; }
        let cidx = color & 0x0F;
        let font = font_bytes();

        let mut pos = x;
        let mut max_pos = x;

        for ch in text.chars() {
            if ch == '\n' {
                if pos > max_pos { max_pos = pos; }
                pos = x;
                y += ADV * scale; // TIC uses TIC_FONT_HEIGHT (6); same as ADV here
                continue;
            }

            let code = (ch as u32 & 0x7F) as usize;
            let base = code * GLYPH_H;
            if base + GLYPH_H > font.len() {
                pos += ADV * scale;
                continue;
            }

            let (start_col, width_cols) = if !fixed {
                let mut left = GLYPH_W;
                let mut right = 0;
                for row in 0..GLYPH_H {
                    let mask = font[base + row];
                    if mask != 0 {
                        let mut l = 0;
                        while l < GLYPH_W && ((mask >> l) & 1) == 0 { l += 1; }
                        let mut r = GLYPH_W;
                        while r > 0 && ((mask >> (r - 1)) & 1) == 0 { r -= 1; }
                        if l < left { left = l; }
                        if r > right { right = r; }
                    }
                }
                let width = right.saturating_sub(left);
                (left, width)
            } else {
                (0, GLYPH_W)
            };

            // Draw glyph
            for row in 0..GLYPH_H {
                let mask = font[base + row];
                for col in 0..width_cols {
                    let bit_idx = start_col + col;
                    if bit_idx < GLYPH_W && ((mask >> bit_idx) & 1) != 0 {
                        let px = pos + (col as i32) * scale;
                        let py = y + (row as i32) * scale;
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let _ = self.pix(px + sx, py + sy, Some(cidx));
                            }
                        }
                    }
                }
            }

            // Advance
            if !fixed {
                if width_cols > 0 {
                    pos += ((width_cols as i32) + 1) * scale;
                } else {
                    pos += ADV * scale;
                }
            } else {
                pos += ADV * scale;
            }
        }

        if pos > max_pos { pos - x } else { max_pos - x }
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
  print("Hello", 10, 10, 15)
  line(0,0,239,135, 14)
  rect(20, 20, 40, 20, 9)
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

            // line(x0,y0,x1,y1,color)
            let fb_line = fb.clone();
            let line_fn = lua.create_function(move |_, (x0, y0, x1, y1, color): (f32, f32, f32, f32, u8)| {
                fb_line.borrow_mut().line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, color);
                Ok(())
            })?;
            globals.set("line", line_fn)?;

            // rect(x,y,w,h,color)
            let fb_rect = fb.clone();
            let rect_fn = lua.create_function(move |_, (x, y, w, h, color): (i32, i32, i32, i32, u8)| {
                fb_rect.borrow_mut().rect(x, y, w, h, color);
                Ok(())
            })?;
            globals.set("rect", rect_fn)?;

            // print(text, x=0, y=0, color=15, fixed=false, scale=1, small=false) -> width
            let fb_print = fb.clone();
            let print_fn = lua.create_function(move |_, args: MultiValue| {
                // Defaults
                let mut text = String::new();
                let mut x: i32 = 0;
                let mut y: i32 = 0;
                let mut color: u8 = 15;
                let mut fixed = false;
                let mut scale: i32 = 1;
                let mut small = false;

                // Parse arguments by position
                for (i, v) in args.iter().enumerate() {
                    match (i, v) {
                        (0, Value::String(s)) => { text = s.to_str()?.to_string(); }
                        (1, Value::Integer(n)) => { x = *n as i32; }
                        (2, Value::Integer(n)) => { y = *n as i32; }
                        (3, Value::Integer(n)) => { color = (*n as i64).clamp(0, 255) as u8; }
                        (4, Value::Boolean(b)) => { fixed = *b; }
                        (5, Value::Integer(n)) => { scale = (*n as i32).max(1); }
                        (6, Value::Boolean(b)) => { small = *b; }
                        _ => {}
                    }
                }

                let width = fb_print.borrow_mut().print_text(&text, x, y, color, fixed, scale, small);
                Ok(width)
            })?;
            globals.set("print", print_fn)?;

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
