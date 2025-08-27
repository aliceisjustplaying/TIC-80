use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::editor::code::{Area, CodeBuffer};
use tic80_rust::gfx::framebuffer::Framebuffer;

#[test]
fn code_view_renders_lines_and_gutter() {
    let text = "line1\nline2\nline3\n";
    let mut cb = CodeBuffer::from_text(text);
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbb = fb.borrow_mut();
    let area = Area {
        x: 0,
        y: 12,
        w: 240,
        h: 40,
    };
    cb.draw(&mut fbb, area);
    // Expect some non-zero pixels in gutter (left side)
    let mut gutter_ink = 0;
    for y in 12..20 {
        for x in 2..22 {
            if fbb.pix(x, y, None).unwrap_or(0) != 0 {
                gutter_ink += 1;
            }
        }
    }
    assert!(gutter_ink > 0);
    // Expect some text pixels in the first line area after gutter (x >= 24)
    let mut line_ink = 0;
    for x in 24..60 {
        if fbb.pix(x, 12, None).unwrap_or(0) != 0 {
            line_ink += 1;
        }
    }
    assert!(line_ink > 0);
}
