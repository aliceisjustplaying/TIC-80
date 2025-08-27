use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use tic80_rust::core::memory::Memory;
use tic80_rust::gfx::framebuffer::Framebuffer;
use tic80_rust::script::lua_runner::{trace_buffer_init, trace_buffer_take, LuaRunner};

#[test]
fn lua_trace_buffers_and_prints() {
    trace_buffer_init();
    let script = r#"
        function BOOT() cls(0) end
        function TIC()
            trace("hello", 7)
        end
    "#;
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    let runner = LuaRunner::new(fb, mem, script).expect("lua init");
    runner.tick();
    let out = trace_buffer_take();
    assert!(
        out.iter().any(|s| s.contains("hello")),
        "expected trace buffer to contain 'hello'"
    );
}

#[test]
fn lua_time_increases_over_real_time() {
    let script = r#"
        tprev = nil
        function BOOT() cls(0) end
        function TIC()
            local t = time()
            if tprev ~= nil and t > tprev + 1 then pix(0,0,9) end
            tprev = t
        end
    "#;
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Rc::new(RefCell::new(Memory::new(fb.clone())));
    let runner = LuaRunner::new(fb.clone(), mem, script).expect("lua init");
    runner.tick();
    thread::sleep(Duration::from_millis(10));
    runner.tick();
    assert_eq!(fb.borrow_mut().pix(0, 0, None), Some(9));
}
