use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::editor::code::{Area, CodeBuffer};
use tic80_rust::gfx::framebuffer::Framebuffer;

// Helper to sample a pixel color from the framebuffer (palette index)
fn px(fb: &mut Framebuffer, x: i32, y: i32) -> u8 {
    fb.pix(x, y, None).unwrap_or(0)
}

#[test]
fn selection_no_seam_between_lines() {
    // Three short lines; select across all so middle seam is exercised.
    let text = "abc\nabc\nabc\n";
    let mut cb = CodeBuffer::from_text(text);

    // Create selection from start of first line to middle of last line
    cb.caret_line = 0;
    cb.caret_col = 0;
    cb.start_selection();
    cb.caret_line = 2;
    cb.caret_col = 2; // up to 'b'

    let fb_rc = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fb = fb_rc.borrow_mut();
    let area = Area { x: 0, y: 7, w: 240, h: 40 };
    cb.draw(&mut fb, area);

    // Pick column 1 (the 'b') well inside selection run
    let gutter_w = 18i32; // plus 1px gap in renderer
    let col_x = gutter_w + 6 + 2; // inside col #1

    // With TIC-80 logic there should be NO black seam between consecutive lines.
    // Our line pitch should be 7; seam of row0 is at y0 + 6 and is covered by row1 fill.
    let row0_y = area.y;
    let seam_y = row0_y + 6; // seam between row0 and row1
    let c = px(&mut fb, col_x, seam_y);
    assert_ne!(
        c, 0,
        "seam must not be black (should be covered by selection fill)"
    );
}

#[test]
fn selection_right_edge_shadow_height_is_7() {
    let text = "abc def\n"; // ensure next char is space so it doesn't overwrite the right-edge shadow
    let mut cb = CodeBuffer::from_text(text);
    // Select first 3 columns on a single line, keep caret away from right edge
    cb.caret_line = 0;
    cb.caret_col = 3;
    cb.start_selection(); // anchor at (0,3)
    cb.caret_line = 0;
    cb.caret_col = 0; // caret at (0,0): selection is (0,3)

    let fb_rc = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fb = fb_rc.borrow_mut();
    let area = Area { x: 0, y: 7, w: 240, h: 20 };
    cb.draw(&mut fb, area);

    let gutter_w = 18i32; // plus 1px gap
    let gap = 1i32;
    let right_edge_x = gutter_w + gap + 3 * 6 - 1; // vertical shadow at right edge of col2
    let base_y = area.y - 1; // selection fill starts at y-1; shadow spans 7 px down from y

    let mut black_count = 0;
    let mut vals = [0u8; 7];
    for dy in 0..7 {
        let c = px(&mut fb, right_edge_x + 1, base_y + 1 + dy);
        vals[dy as usize] = c;
        if c == 0 {
            black_count += 1;
        }
    }
    eprintln!("right edge column vals={:?}", vals);
    assert_eq!(
        black_count, 7,
        "right-edge shadow must be exactly 7 px tall"
    );
}
