use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use mlua::{Function, Lua, MultiValue, RegistryKey, Result as LuaResult, Value};

use crate::audio::fft::{get_global_fft, query_fft};
use crate::audio::vqt::get_global_vqt;
use crate::core::memory::Memory;
use crate::gfx::framebuffer::Framebuffer;

pub struct LuaRunner {
    lua: Lua,
    tic_key: Option<RegistryKey>,
}

// Optional trace buffer (used by tests); if present, trace() will also append messages here.
static TRACE_BUFFER: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

impl LuaRunner {
    pub fn new(
        fb: Rc<RefCell<Framebuffer>>,
        mem: Rc<RefCell<Memory>>,
        script_src: &str,
    ) -> LuaResult<Self> {
        let lua = Lua::new();
        let start_time = Instant::now();
        let tic_key = {
            let globals = lua.globals();

            // cls(color)
            let fb_cls = fb.clone();
            let cls_fn = lua.create_function(move |_, color: Option<u8>| {
                fb_cls.borrow_mut().cls(color.unwrap_or(0));
                Ok(())
            })?;
            globals.set("cls", cls_fn)?;

            // pix(x,y[,color])
            let fb_pix = fb.clone();
            let pix_fn = lua.create_function(move |_, (x, y, color): (i32, i32, Option<u8>)| {
                let res = fb_pix.borrow_mut().pix(x, y, color);
                Ok(res)
            })?;
            globals.set("pix", pix_fn)?;

            // line(x0,y0,x1,y1,color)
            let fb_line = fb.clone();
            let line_fn = lua.create_function(
                move |_, (x0, y0, x1, y1, color): (f32, f32, f32, f32, u8)| {
                    fb_line
                        .borrow_mut()
                        .line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, color);
                    Ok(())
                },
            )?;
            globals.set("line", line_fn)?;

            // rect(x,y,w,h,color)
            let fb_rect = fb.clone();
            let rect_fn =
                lua.create_function(move |_, (x, y, w, h, color): (i32, i32, i32, i32, u8)| {
                    fb_rect.borrow_mut().rect(x, y, w, h, color);
                    Ok(())
                })?;
            globals.set("rect", rect_fn)?;

            // rectb(x,y,w,h,color)
            let fb_rectb = fb.clone();
            let rectb_fn =
                lua.create_function(move |_, (x, y, w, h, color): (i32, i32, i32, i32, u8)| {
                    fb_rectb.borrow_mut().rectb(x, y, w, h, color);
                    Ok(())
                })?;
            globals.set("rectb", rectb_fn)?;

            // circ(x, y, r, color)
            let fb_circ = fb.clone();
            let circ_fn =
                lua.create_function(move |_, (x, y, r, color): (i32, i32, i32, u8)| {
                    fb_circ.borrow_mut().circ(x, y, r, color);
                    Ok(())
                })?;
            globals.set("circ", circ_fn)?;

            // circb(x, y, r, color)
            let fb_circb = fb.clone();
            let circb_fn =
                lua.create_function(move |_, (x, y, r, color): (i32, i32, i32, u8)| {
                    fb_circb.borrow_mut().circb(x, y, r, color);
                    Ok(())
                })?;
            globals.set("circb", circb_fn)?;

            // clip(x,y,w,h) or clip() to reset
            let fb_clip = fb.clone();
            let clip_fn = lua.create_function(move |_, args: MultiValue| {
                if args.is_empty() {
                    fb_clip.borrow_mut().clip_reset();
                } else {
                    let x = match args.get(0) {
                        Some(Value::Integer(n)) => *n as i32,
                        _ => 0,
                    };
                    let y = match args.get(1) {
                        Some(Value::Integer(n)) => *n as i32,
                        _ => 0,
                    };
                    let w = match args.get(2) {
                        Some(Value::Integer(n)) => *n as i32,
                        _ => 0,
                    };
                    let h = match args.get(3) {
                        Some(Value::Integer(n)) => *n as i32,
                        _ => 0,
                    };
                    fb_clip.borrow_mut().clip(x, y, w, h);
                }
                Ok(())
            })?;
            globals.set("clip", clip_fn)?;

            // print(text, x=0, y=0, color=15, fixed=false, scale=1, small=false) -> width
            #[derive(Default)]
            struct PrintArgs {
                text: String,
                x: i32,
                y: i32,
                color: u8,
                fixed: bool,
                scale: i32,
                small: bool,
            }

            impl PrintArgs {
                fn from_lua(args: &MultiValue) -> LuaResult<Self> {
                    let mut out = PrintArgs {
                        color: 15,
                        scale: 1,
                        ..Default::default()
                    };
                    for (i, v) in args.iter().enumerate() {
                        match (i, v) {
                            (0, Value::String(s)) => out.text = s.to_str()?.to_string(),
                            (1, Value::Integer(n)) => out.x = *n as i32,
                            (2, Value::Integer(n)) => out.y = *n as i32,
                            (3, Value::Integer(n)) => out.color = (*n).clamp(0, 255) as u8,
                            (4, Value::Boolean(b)) => out.fixed = *b,
                            (5, Value::Integer(n)) => out.scale = (*n as i32).max(1),
                            (6, Value::Boolean(b)) => out.small = *b,
                            _ => {}
                        }
                    }
                    Ok(out)
                }
            }

            let fb_print = fb.clone();
            let print_fn = lua.create_function(move |_, args: MultiValue| {
                let p = PrintArgs::from_lua(&args)?;
                let width = fb_print
                    .borrow_mut()
                    .print_text(&p.text, p.x, p.y, p.color, p.fixed, p.scale, p.small);
                Ok(width)
            })?;
            globals.set("print", print_fn)?;

            // FFT APIs: fft/ffts/fftr/fftrs
            fn parse_fft_args(args: &MultiValue) -> (i32, i32) {
                let start = match args.get(0) {
                    Some(Value::Integer(n)) => *n as i32,
                    _ => -1,
                };
                let end = match args.get(1) {
                    Some(Value::Integer(n)) => *n as i32,
                    _ => -1,
                };
                (start, end)
            }

            let fft_fn = lua.create_function(move |_, args: MultiValue| {
                let (start, end) = parse_fft_args(&args);
                let val = if let Some(arc) = get_global_fft() {
                    let guard = arc.read();
                    query_fft(&guard, start, end, false, false)
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("fft", fft_fn)?;

            let ffts_fn = lua.create_function(move |_, args: MultiValue| {
                let (start, end) = parse_fft_args(&args);
                let val = if let Some(arc) = get_global_fft() {
                    let guard = arc.read();
                    query_fft(&guard, start, end, true, false)
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("ffts", ffts_fn)?;

            let fftr_fn = lua.create_function(move |_, args: MultiValue| {
                let (start, end) = parse_fft_args(&args);
                let val = if let Some(arc) = get_global_fft() {
                    let guard = arc.read();
                    query_fft(&guard, start, end, false, true)
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("fftr", fftr_fn)?;

            let fftrs_fn = lua.create_function(move |_, args: MultiValue| {
                let (start, end) = parse_fft_args(&args);
                let val = if let Some(arc) = get_global_fft() {
                    let guard = arc.read();
                    query_fft(&guard, start, end, true, true)
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("fftrs", fftrs_fn)?;

            // VQT APIs: vqt/vqts/vqtr/vqtrs and whitened variants vqtw/vqtsw/vqtrw/vqtrsw
            let vqt_fn = lua.create_function(move |_, bin: i32| {
                let val = if let Some(arc) = get_global_vqt() {
                    let guard = arc.read();
                    // Return normalized instantaneous; align with TIC: vqt returns normalized
                    if bin >= 0 && (bin as usize) < guard.bins_count() {
                        guard.vqt_norm[bin as usize] as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("vqt", vqt_fn)?;

            let vqts_fn = lua.create_function(move |_, bin: i32| {
                let val = if let Some(arc) = get_global_vqt() {
                    let guard = arc.read();
                    if bin >= 0 && (bin as usize) < guard.bins_count() {
                        guard.vqt_norm[bin as usize] as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("vqts", vqts_fn)?;

            let vqtr_fn = lua.create_function(move |_, bin: i32| {
                let val = if let Some(arc) = get_global_vqt() {
                    let guard = arc.read();
                    if bin >= 0 && (bin as usize) < guard.bins_count() {
                        guard.vqt_raw[bin as usize] as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("vqtr", vqtr_fn)?;

            let vqtrs_fn = lua.create_function(move |_, bin: i32| {
                let val = if let Some(arc) = get_global_vqt() {
                    let guard = arc.read();
                    if bin >= 0 && (bin as usize) < guard.bins_count() {
                        guard.vqt_sm[bin as usize] as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("vqtrs", vqtrs_fn)?;

            let vqtw_fn = lua.create_function(move |_, bin: i32| {
                let val = if let Some(arc) = get_global_vqt() {
                    let guard = arc.read();
                    if bin >= 0 && (bin as usize) < guard.bins_count() {
                        guard.vqt_w_norm[bin as usize] as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("vqtw", vqtw_fn)?;

            let vqtsw_fn = lua.create_function(move |_, bin: i32| {
                let val = if let Some(arc) = get_global_vqt() {
                    let guard = arc.read();
                    if bin >= 0 && (bin as usize) < guard.bins_count() {
                        guard.vqt_w_norm[bin as usize] as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("vqtsw", vqtsw_fn)?;

            let vqtrw_fn = lua.create_function(move |_, bin: i32| {
                let val = if let Some(arc) = get_global_vqt() {
                    let guard = arc.read();
                    if bin >= 0 && (bin as usize) < guard.bins_count() {
                        guard.vqt_w_raw[bin as usize] as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("vqtrw", vqtrw_fn)?;

            let vqtrsw_fn = lua.create_function(move |_, bin: i32| {
                let val = if let Some(arc) = get_global_vqt() {
                    let guard = arc.read();
                    if bin >= 0 && (bin as usize) < guard.bins_count() {
                        guard.vqt_w_sm[bin as usize] as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                Ok(val)
            })?;
            globals.set("vqtrsw", vqtrsw_fn)?;

            // trace(message, color=15)
            let trace_fn = lua.create_function(move |_, args: MultiValue| {
                let msg = match args.get(0) {
                    Some(Value::String(s)) => s.to_str()?.to_string(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Integer(i)) => i.to_string(),
                    Some(Value::Boolean(b)) => b.to_string(),
                    Some(Value::Nil) | None => String::new(),
                    _ => String::new(),
                };
                let color = match args.get(1) {
                    Some(Value::Integer(n)) => *n as i32,
                    Some(Value::Number(n)) => *n as i32,
                    _ => 15,
                };
                // Print to console; color is informational only here.
                println!("[trace:{}] {}", color, msg);
                if let Some(buf) = TRACE_BUFFER.get() {
                    if let Ok(mut b) = buf.lock() {
                        b.push(msg);
                    }
                }
                Ok(())
            })?;
            globals.set("trace", trace_fn)?;

            // time() -> milliseconds since cart start
            let start_copy = start_time;
            let time_fn = lua.create_function(move |_, ()| {
                let ms = start_copy.elapsed().as_millis() as f64;
                Ok(ms)
            })?;
            globals.set("time", time_fn)?;

            // memory: peek/poke + bit variants + memcpy/memset
            let mem_peek = mem.clone();
            let peek_fn = lua.create_function(move |_, (addr, bits): (u32, Option<u8>)| {
                let a = addr as usize;
                let b = bits.unwrap_or(8);
                let v = if b == 8 {
                    mem_peek.borrow().peek(a)
                } else {
                    mem_peek.borrow().peek_bits(a, b)
                };
                Ok(v as u32)
            })?;
            globals.set("peek", peek_fn)?;

            let mem_poke = mem.clone();
            let poke_fn =
                lua.create_function(move |_, (addr, val, bits): (u32, u32, Option<u8>)| {
                    let a = addr as usize;
                    let v = val as u8;
                    let b = bits.unwrap_or(8);
                    if b == 8 {
                        mem_poke.borrow_mut().poke(a, v);
                    } else {
                        mem_poke.borrow_mut().poke_bits(a, b, v);
                    }
                    Ok(())
                })?;
            globals.set("poke", poke_fn)?;

            let mem_peek1 = mem.clone();
            globals.set(
                "peek1",
                lua.create_function(move |_, addr: u32| {
                    Ok(mem_peek1.borrow().peek_bits(addr as usize, 1) as u32)
                })?,
            )?;
            let mem_peek2 = mem.clone();
            globals.set(
                "peek2",
                lua.create_function(move |_, addr: u32| {
                    Ok(mem_peek2.borrow().peek_bits(addr as usize, 2) as u32)
                })?,
            )?;
            let mem_peek4 = mem.clone();
            globals.set(
                "peek4",
                lua.create_function(move |_, addr: u32| {
                    Ok(mem_peek4.borrow().peek_bits(addr as usize, 4) as u32)
                })?,
            )?;

            let mem_poke1 = mem.clone();
            globals.set(
                "poke1",
                lua.create_function(move |_, (addr, val): (u32, u32)| {
                    mem_poke1
                        .borrow_mut()
                        .poke_bits(addr as usize, 1, val as u8);
                    Ok(())
                })?,
            )?;
            let mem_poke2 = mem.clone();
            globals.set(
                "poke2",
                lua.create_function(move |_, (addr, val): (u32, u32)| {
                    mem_poke2
                        .borrow_mut()
                        .poke_bits(addr as usize, 2, val as u8);
                    Ok(())
                })?,
            )?;
            let mem_poke4 = mem.clone();
            globals.set(
                "poke4",
                lua.create_function(move |_, (addr, val): (u32, u32)| {
                    mem_poke4
                        .borrow_mut()
                        .poke_bits(addr as usize, 4, val as u8);
                    Ok(())
                })?,
            )?;

            let mem_memcpy = mem.clone();
            globals.set(
                "memcpy",
                lua.create_function(move |_, (dst, src, size): (u32, u32, u32)| {
                    mem_memcpy
                        .borrow_mut()
                        .memcpy(dst as usize, src as usize, size as usize);
                    Ok(())
                })?,
            )?;
            let mem_memset = mem.clone();
            globals.set(
                "memset",
                lua.create_function(move |_, (dst, val, size): (u32, u32, u32)| {
                    mem_memset
                        .borrow_mut()
                        .memset(dst as usize, val as u8, size as usize);
                    Ok(())
                })?,
            )?;

            // elli(x, y, a, b, color)
            let fb_elli = fb.clone();
            let elli_fn =
                lua.create_function(move |_, (x, y, a, b, color): (i32, i32, i32, i32, u8)| {
                    fb_elli.borrow_mut().elli(x, y, a, b, color);
                    Ok(())
                })?;
            globals.set("elli", elli_fn)?;

            // ellib(x, y, a, b, color)
            let fb_ellib = fb.clone();
            let ellib_fn =
                lua.create_function(move |_, (x, y, a, b, color): (i32, i32, i32, i32, u8)| {
                    fb_ellib.borrow_mut().ellib(x, y, a, b, color);
                    Ok(())
                })?;
            globals.set("ellib", ellib_fn)?;

            // tri(x1,y1,x2,y2,x3,y3,color)
            let fb_tri = fb.clone();
            let tri_fn = lua.create_function(
                move |_, (x1, y1, x2, y2, x3, y3, color): (i32, i32, i32, i32, i32, i32, u8)| {
                    fb_tri.borrow_mut().tri(x1, y1, x2, y2, x3, y3, color);
                    Ok(())
                },
            )?;
            globals.set("tri", tri_fn)?;

            // trib(x1,y1,x2,y2,x3,y3,color)
            let fb_trib = fb.clone();
            let trib_fn = lua.create_function(
                move |_, (x1, y1, x2, y2, x3, y3, color): (i32, i32, i32, i32, i32, i32, u8)| {
                    fb_trib.borrow_mut().trib(x1, y1, x2, y2, x3, y3, color);
                    Ok(())
                },
            )?;
            globals.set("trib", trib_fn)?;

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

    pub fn tick(&self) {
        if let Some(key) = &self.tic_key {
            if let Ok(func) = self.lua.registry_value::<Function>(key) {
                let _ = func.call::<_, ()>(());
            }
        }
    }
}

// Test support: initialize a shared trace buffer (clears any existing messages).
pub fn trace_buffer_init() {
    let _ = TRACE_BUFFER.set(Mutex::new(Vec::new()));
    if let Some(b) = TRACE_BUFFER.get() {
        if let Ok(mut v) = b.lock() {
            v.clear();
        }
    }
}

// Test support: take and clear all trace messages.
pub fn trace_buffer_take() -> Vec<String> {
    if let Some(b) = TRACE_BUFFER.get() {
        if let Ok(mut v) = b.lock() {
            let out = v.clone();
            v.clear();
            return out;
        }
    }
    Vec::new()
}
