use std::cell::RefCell;
use std::rc::Rc;

use mlua::{Function, Lua, MultiValue, RegistryKey, Result as LuaResult, Value};

use crate::gfx::framebuffer::Framebuffer;

pub struct LuaRunner {
    lua: Lua,
    tic_key: Option<RegistryKey>,
}

impl LuaRunner {
    pub fn new(fb: Rc<RefCell<Framebuffer>>, script_src: &str) -> LuaResult<Self> {
        let lua = Lua::new();
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
            let rectb_fn = lua.create_function(
                move |_, (x, y, w, h, color): (i32, i32, i32, i32, u8)| {
                    fb_rectb.borrow_mut().rectb(x, y, w, h, color);
                    Ok(())
                },
            )?;
            globals.set("rectb", rectb_fn)?;

            // clip(x,y,w,h) or clip() to reset
            let fb_clip = fb.clone();
            let clip_fn = lua.create_function(move |_, args: MultiValue| {
                if args.is_empty() {
                    fb_clip.borrow_mut().clip_reset();
                } else {
                    let x = match args.get(0) { Some(Value::Integer(n)) => *n as i32, _ => 0 };
                    let y = match args.get(1) { Some(Value::Integer(n)) => *n as i32, _ => 0 };
                    let w = match args.get(2) { Some(Value::Integer(n)) => *n as i32, _ => 0 };
                    let h = match args.get(3) { Some(Value::Integer(n)) => *n as i32, _ => 0 };
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
