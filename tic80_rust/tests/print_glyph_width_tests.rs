use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::gfx::framebuffer::Framebuffer;

#[test]
fn variable_width_trimming_i_vs_m() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let wi = fbm.print_text("i", 0, 0, 1, false, 1, false);
    let wm = fbm.print_text("m", 0, 10, 1, false, 1, false);
    assert!(wi < wm);
}

#[test]
fn fixed_width_monospace_equal_width() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let w1 = fbm.print_text("i", 0, 0, 1, true, 1, false);
    let w2 = fbm.print_text("m", 0, 10, 1, true, 1, false);
    let w3 = fbm.print_text("!", 0, 20, 1, true, 1, false);
    assert_eq!(w1, w2);
    assert_eq!(w2, w3);
}

#[test]
fn punctuation_width_and_spacing() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    // Variable width should still advance by width+1 column for inter-glyph spacing
    let w1 = fbm.print_text("!", 10, 10, 1, false, 1, false);
    let w2 = fbm.print_text("!!", 10, 20, 1, false, 1, false);
    assert!(w2 >= w1 * 2 - 1); // allow for glyph trim; spacing yields near double width
}

#[test]
fn multiline_width_is_max_line_width() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let w = fbm.print_text("mmmm\nmm", 5, 5, 1, false, 1, false);
    // First line longer than second; width should reflect the first line
    let w_first = fbm.print_text("mmmm", 0, 0, 1, false, 1, false);
    assert_eq!(w, w_first);
}
