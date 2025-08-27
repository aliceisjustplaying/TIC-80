use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::core::memory::Memory;
use tic80_rust::gfx::framebuffer::Framebuffer;
use tic80_rust::script::lua_runner::LuaRunner;

#[test]
fn lua_line_float_truncates_to_integer_pixels() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    // Lua script draws a vertical line at x=1.9 (~x=1) from y=0.0 to y=5.9 (~y=5)
    let script = r#"
        function TIC()
            cls(0)
            line(1.9, 0.0, 1.9, 5.9, 3)
        end
    "#;
    let runner = LuaRunner::new(fb.clone(), mem.clone(), script).unwrap();
    runner.tick();
    // Expect pixels at x=1, y in [0..5] to be colored, and x=2 blank in that range
    for y in 0..6 {
        assert_eq!(fb.borrow_mut().pix(1, y, None), Some(3));
        assert_eq!(fb.borrow_mut().pix(2, y, None), Some(0));
    }
}
