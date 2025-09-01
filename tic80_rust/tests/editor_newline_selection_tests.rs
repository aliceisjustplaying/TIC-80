use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::editor::code::{Area, CodeBuffer};
use tic80_rust::gfx::framebuffer::Framebuffer;

#[test]
fn selection_covers_newline_cell_on_empty_line() {
    // Line 1 is empty between a and b
    let text = "a\n\nb\n";
    let mut cb = CodeBuffer::from_text(text);
    // Select from end of first line through the empty line
    cb.caret_line = 0;
    cb.caret_col = 1; // after 'a'
    cb.start_selection();
    cb.caret_line = 2;
    cb.caret_col = 0; // before 'b'

    let fb_rc = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fb = fb_rc.borrow_mut();
    let area = Area { x: 0, y: 7, w: 240, h: 40 };
    cb.draw(&mut fb, area);

    // Expect a selection cell drawn on the empty line at column 0 (gutter=18 + gap=1)
    let gutter = 18i32;
    let gap = 1i32;
    let x = gutter + gap;
    let y = area.y + 7; // second row (empty line)
    let c = fb.pix(x, y, None).unwrap_or(0);
    assert_ne!(c, 0, "expected selection cell on empty line for newline");
}
