use std::cell::RefCell;
use std::rc::Rc;
use std::fs;
use std::path::PathBuf;

use tic80_rust::gfx::framebuffer::{dimensions, Framebuffer};
use tic80_rust::script::lua_runner::LuaRunner;
use tic80_rust::core::memory::Memory;

fn run_lua(script: &str, ticks: usize) -> Rc<RefCell<Framebuffer>> {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    let runner = LuaRunner::new(fb.clone(), mem, script).expect("lua init");
    for _ in 0..ticks { runner.tick(); }
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

#[test]
fn lua_runs_alt_cart_file() {
    // Load the alternate cart file from assets and run one tick
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("assets/alt.lua");
    let script = fs::read_to_string(&path).expect("read alt.lua");
    let fb = run_lua(&script, 1);
    let mut fbm = fb.borrow_mut();
    // Background set in BOOT
    assert_eq!(fbm.pix(20, 20, None), Some(2));
    // Marker at top-left and filled square
    assert_eq!(fbm.pix(0, 0, None), Some(7));
    assert_eq!(fbm.pix(9, 9, None), Some(5));
}

#[test]
fn lua_clip_and_rectb() {
    let script = r#"
        function BOOT()
            cls(1)
            clip(0, 0, 1, 1)
        end
        function TIC()
            rectb(0, 0, 3, 3, 7)
        end
    "#;
    let fb = run_lua(script, 1);
    let mut fbm = fb.borrow_mut();
    // Only origin affected due to clipping; neighbors unchanged
    assert_eq!(fbm.pix(0, 0, None), Some(7));
    assert_eq!(fbm.pix(1, 0, None), Some(1));
    assert_eq!(fbm.pix(0, 1, None), Some(1));
}

#[test]
fn lua_pix_oob_read_returns_nil() {
    let script = r#"
        function BOOT() cls(0) end
        function TIC()
            local ok = true
            if pix(-1, 0) ~= nil then ok = false end
            if pix(0, -1) ~= nil then ok = false end
            if pix(240, 0) ~= nil then ok = false end
            if pix(0, 136) ~= nil then ok = false end
            if ok then pix(0, 0, 5) end
        end
    "#;
    let fb = run_lua(script, 1);
    // Marker set only if nil checks passed
    assert_eq!(fb.borrow_mut().pix(0, 0, None), Some(5));
}

fn fb_hash(fb: &mut Framebuffer) -> u64 {
    let (w, h) = dimensions();
    let mut hash: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;
    for y in 0..(h as i32) {
        for x in 0..(w as i32) {
            let b = fb.pix(x, y, None).unwrap_or(0);
            hash ^= b as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}

#[test]
fn lua_default_cart_deterministic_hash() {
    // Load default cart and run N frames twice; hashes should match
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("assets/default.lua");
    let script = std::fs::read_to_string(&path).expect("read default.lua");

    let run_hash = |ticks: usize| -> u64 {
        let fb = Rc::new(RefCell::new(Framebuffer::new()));
        let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
        let runner = LuaRunner::new(fb.clone(), mem, &script).expect("lua init");
        for _ in 0..ticks { runner.tick(); }
        let mut borrowed = fb.borrow_mut();
        fb_hash(&mut borrowed)
    };

    let h1 = run_hash(1);
    let h1_again = run_hash(1);
    assert_eq!(h1, h1_again, "hash should be deterministic for 1 tick");

    let h2 = run_hash(2);
    let h2_again = run_hash(2);
    assert_eq!(h2, h2_again, "hash should be deterministic for 2 ticks");

    // Different frame counts should usually yield different hashes for this cart
    assert_ne!(h1, h2, "different ticks should yield different frame hashes");
}

#[test]
fn lua_circ_and_circb() {
    let script = r#"
        function BOOT() cls(0) end
        function TIC()
            circ(20, 20, 4, 6)
            circb(30, 20, 3, 9)
        end
    "#;
    let fb = run_lua(script, 1);
    let mut fbm = fb.borrow_mut();
    // Filled circle: center row span for r=4
    for x in 16..=24 { assert_eq!(fbm.pix(x, 20, None), Some(6)); }
    // Border circle: cardinal points for r=3
    assert_eq!(fbm.pix(33, 20, None), Some(9));
    assert_eq!(fbm.pix(27, 20, None), Some(9));
    assert_eq!(fbm.pix(30, 23, None), Some(9));
    assert_eq!(fbm.pix(30, 17, None), Some(9));
}

#[test]
fn lua_elli_ellib_and_tri_trib() {
    let script = r#"
        function BOOT() cls(0) end
        function TIC()
            elli(60, 20, 5, 3, 4)
            ellib(60, 20, 5, 3, 12)
            tri(80, 10, 90, 20, 70, 20, 6)
            trib(100, 10, 110, 20, 90, 20, 9)
        end
    "#;
    let fb = run_lua(script, 1);
    let mut fbm = fb.borrow_mut();
    // Ellipse border cardinals
    assert_eq!(fbm.pix(65, 20, None), Some(12));
    assert_eq!(fbm.pix(55, 20, None), Some(12));
    assert_eq!(fbm.pix(60, 23, None), Some(12));
    assert_eq!(fbm.pix(60, 17, None), Some(12));
    // Filled ellipse center row (interior only; endpoints are border color)
    for x in 56..=64 { assert_eq!(fbm.pix(x, 20, None), Some(4)); }
    // Triangle interior
    assert_eq!(fbm.pix(80, 18, None), Some(6));
    // Border triangle vertices
    assert_eq!(fbm.pix(100, 10, None), Some(9));
    assert_eq!(fbm.pix(110, 20, None), Some(9));
    assert_eq!(fbm.pix(90, 20, None), Some(9));
}
