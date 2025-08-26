use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::gfx::framebuffer::{dimensions, Framebuffer};
use tic80_rust::script::lua_runner::LuaRunner;

fn run_lua(script: &str, ticks: usize) -> Rc<RefCell<Framebuffer>> {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let runner = LuaRunner::new(fb.clone(), script).expect("lua init");
    for _ in 0..ticks {
        runner.tick();
    }
    fb
}

#[test]
fn lua_cls_and_pix() {
    let script = r#"
        function BOOT()
            cls(2)
        end
        function TIC()
            pix(1, 1, 9)
        end
    "#;
    let fb = run_lua(script, 1);
    assert_eq!(fb.borrow_mut().pix(1, 1, None), Some(9));
    assert_eq!(fb.borrow_mut().pix(0, 0, None), Some(2));
}

#[test]
fn lua_line_and_rect() {
    let script = r#"
        function BOOT()
            cls(0)
        end
        function TIC()
            line(0, 0, 2, 0, 5)
            rect(0, 1, 3, 1, 6)
        end
    "#;
    let fb = run_lua(script, 1);
    let mut fbm = fb.borrow_mut();
    // line
    assert_eq!(fbm.pix(0, 0, None), Some(5));
    assert_eq!(fbm.pix(1, 0, None), Some(5));
    assert_eq!(fbm.pix(2, 0, None), Some(5));
    // rect one row below
    assert_eq!(fbm.pix(0, 1, None), Some(6));
    assert_eq!(fbm.pix(1, 1, None), Some(6));
    assert_eq!(fbm.pix(2, 1, None), Some(6));
}

#[test]
fn lua_print_width_marker() {
    let script = r#"
        function BOOT()
            cls(0)
        end
        function TIC()
            local w = print("AB", 10, 10, 1, false, 1, false)
            pix(10 + w, 10, 14)
        end
    "#;
    let fb = run_lua(script, 1);
    let mut fbm = fb.borrow_mut();
    // text drew something in the glyph area near (10,10)
    let mut any = false;
    for yy in 10..(10 + 8) {
        for xx in 10..(10 + 8) {
            if fbm.pix(xx, yy, None) == Some(1) {
                any = true;
                break;
            }
        }
        if any { break; }
    }
    assert!(any, "expected some glyph pixels drawn near (10,10)");

    // marker at x+width somewhere on row y=10
    let (w, _) = dimensions();
    let mut found_marker = false;
    for x in 10..(w as i32) {
        if fbm.pix(x, 10, None) == Some(14) { found_marker = true; break; }
    }
    assert!(found_marker, "expected marker pixel with color 14 on row 10");
}

#[test]
fn lua_print_defaults_and_pix_read() {
    let script = r#"
        function BOOT()
            cls(0)
        end
        function TIC()
            local w = print("A")
            -- Strict gating: require that at least one glyph pixel was drawn
            -- near the origin using the default color (15), rather than probing (0,0).
            local drawn = false
            for yy = 0, 7 do
                for xx = 0, 7 do
                    if pix(xx, yy) == 15 then drawn = true break end
                end
                if drawn then break end
            end
            if drawn then pix(w, 0, 7) end
        end
    "#;
    let fb = run_lua(script, 1);
    let mut fbm = fb.borrow_mut();
    // default color drew some glyph pixels near origin
    let mut any = false;
    for yy in 0..8 {
        for xx in 0..8 {
            if fbm.pix(xx, yy, None) == Some(15) { any = true; break; }
        }
        if any { break; }
    }
    assert!(any, "expected some glyph pixels drawn near origin");
    // verify the script marked (w,0) with 7 (we don't need to know w here)
    // find first non-zero on row 0 after x=0
    let (w, _) = dimensions();
    let mut found = false;
    for x in 1..(w as i32) {
        if fbm.pix(x, 0, None) == Some(7) {
            found = true;
            break;
        }
    }
    assert!(found, "expected a marker pixel with color 7 on row 0");
}
