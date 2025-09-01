use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::editor::code::{Area, CodeBuffer};
use tic80_rust::gfx::framebuffer::Framebuffer;

// Verify selection top aligns with caret fill top (one px above text baseline)
#[test]
fn selection_aligned_with_caret_box() {
    let text = "a  def\n";
    let mut cb = CodeBuffer::from_text(text);
    // Prepare selection from col 1 to col 3 on line 0
    cb.caret_line = 0;
    cb.caret_col = 1;
    cb.start_selection();
    cb.caret_col = 3; // selection [1,3)
                      // Also place caret for box at col 3
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbb = fb.borrow_mut();
    let area = Area {
        x: 0,
        y: 12,
        w: 240,
        h: 124,
    };
    cb.draw(&mut fbb, area);

    // Expected coordinates
    let gutter_w = 24i32;
    let row = 0i32; // first line
                    // updated code view line pitch is 7 px
    let gutter_y = area.y + row * 7;
    let caret_fill_top = (gutter_y - 1).max(area.y); // caret fills 7px starting 1px above baseline, clipped to area
                                                     // Selection starts at col 1 (from) over a space (no glyph ink)
    let sel_x = area.x + gutter_w + 6;
    let sel_y = caret_fill_top;

    // Sample a pixel inside selection highlight
    let c = fbb.pix(sel_x + 1, sel_y, None).unwrap_or(0);
    assert_ne!(c, 0, "expected selection overlay at aligned top row");
}
