use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::editor::ui::{EditorUi, Tab};
use tic80_rust::gfx::framebuffer::Framebuffer;

#[test]
fn editor_draws_top_bar_and_switches_tabs() {
    let mut ui = EditorUi::new(3.0);
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    {
        let mut fbb = fb.borrow_mut();
        ui.draw(&mut fbb);
        // Top bar pixel should be non-zero (colored)
        assert!(fbb.pix(1, 1, None).unwrap_or(0) != 0);
    }
    // Click on CONSOLE tab
    ui.on_click_fb(50, 5);
    assert_eq!(ui.active, Tab::Console);
    // Click back to CODE
    ui.on_click_fb(6, 5);
    assert_eq!(ui.active, Tab::Code);
}
