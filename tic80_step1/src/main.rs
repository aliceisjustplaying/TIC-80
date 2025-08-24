use mlua::prelude::*;
use sdl2::pixels::Color;
use std::sync::{Arc, Mutex};

// The absolute minimum state: a framebuffer and a palette.
pub struct TicCore {
    vram: [u8; 240 * 136 / 2], // 4-bit pixels
    palette: [[u8; 4]; 16],    // RGBA colors
}

// The `pix` function is the heart of this step.
fn register_api(lua: &Lua, core: Arc<Mutex<TicCore>>) -> LuaResult<()> {
    let globals = lua.globals();

    let core_clone = Arc::clone(&core);
    let pix = lua.create_function_mut(move |_, (x, y, color): (i32, i32, u8)| {
        // Bounds check
        if x < 0 || x >= 240 || y < 0 || y >= 136 {
            return Ok(());
        }

        let mut core = core_clone.lock().unwrap();
        let color = color & 0x0F; // Ensure it's a 4-bit color

        // The VRAM is packed, so we need to do some bit manipulation.
        let index = (y as usize * 240 + x as usize) / 2;
        if index >= core.vram.len() {
            return Ok(());
        }
        let byte = &mut core.vram[index];

        if x % 2 == 0 {
            // Even pixel, change the low nibble
            *byte = (*byte & 0xF0) | color;
        } else {
            // Odd pixel, change the high nibble
            *byte = (*byte & 0x0F) | (color << 4);
        }

        Ok(())
    })?;

    globals.set("pix", pix)?;
    Ok(())
}

fn main() {
    // --- SDL2 Setup ---
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("TIC-80 Rust (Step 1)", 240 * 3, 136 * 3) // Scaled up
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // 1. Create the core state, wrapped for sharing.
    let core = Arc::new(Mutex::new(TicCore {
        vram: [0; 240 * 136 / 2],
        // Hardcode the default TIC-80 palette
        palette: [
            [0x1a, 0x1c, 0x2c, 0xff], [0x5d, 0x27, 0x5d, 0xff], [0xb1, 0x3e, 0x53, 0xff], [0xef, 0x7d, 0x57, 0xff],
            [0xff, 0xcd, 0x75, 0xff], [0xa7, 0xf0, 0x70, 0xff], [0x38, 0xb7, 0x64, 0xff], [0x25, 0x71, 0x79, 0xff],
            [0x29, 0x36, 0x6f, 0xff], [0x3b, 0x5d, 0xc9, 0xff], [0x41, 0xa6, 0xf6, 0xff], [0x73, 0xef, 0xf7, 0xff],
            [0xf4, 0xf4, 0xf4, 0xff], [0x94, 0xb0, 0xc2, 0xff], [0x56, 0x6c, 0x86, 0xff], [0x33, 0x3c, 0x57, 0xff],
        ],
    }));

    // 2. Initialize Lua and register our single `pix` function.
    let lua = Lua::new();
    register_api(&lua, Arc::clone(&core)).unwrap();

    // 3. Define a hardcoded script. This is our "cartridge".
    const SCRIPT: &str = r#"
function TIC()
    pix(115, 63, 12) -- Draw a single white pixel near the center
end
"#;
    lua.load(SCRIPT).exec().unwrap();
    let tic_func: LuaFunction = lua.globals().get("TIC").unwrap();

    // 4. The Main Loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGBA8888, 240, 136)
        .unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            // Handle closing the window
            use sdl2::event::Event;
            if let Event::Quit { .. } = event {
                break 'running;
            }
        }

        // --- LOGIC ---
        // Call the Lua TIC() function every frame.
        tic_func.call::<_, ()>(()).unwrap();

        // --- RENDER ---
        let core_locked = core.lock().unwrap();
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..136 {
                for x in 0..240 {
                    let index = (y * 240 + x) / 2;
                    let byte = core_locked.vram[index];
                    let color_index = if x % 2 == 0 { byte & 0x0F } else { byte >> 4 };
                    
                    let rgba = core_locked.palette[color_index as usize];
                    let offset = y * pitch + x * 4;
                    buffer[offset..offset + 4].copy_from_slice(&rgba);
                }
            }
        }).unwrap();
        
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
}
