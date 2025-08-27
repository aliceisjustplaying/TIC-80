use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::gfx::framebuffer::Framebuffer;

fn count_drawn_in_col(fb: &mut Framebuffer, x: i32, y: i32, h: i32) -> usize {
    let mut c = 0;
    for yy in y..y + h {
        if fb.pix(x, yy, None).unwrap_or(0) != 0 {
            c += 1;
        }
    }
    c
}

#[test]
fn print_fixed_width_and_return_value() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let x = 10;
    let y = 10;
    let scale = 1;
    // Fixed width: width should be ADV(6) * len * scale
    let text = "im i";
    let w = fbm.print_text(text, x, y, 1, true, scale, false);
    assert_eq!(w, 6 * (text.len() as i32) * scale);
}

#[test]
fn print_variable_space_returns_adv() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let w = fbm.print_text(" ", 0, 0, 1, false, 1, false);
    assert_eq!(w, 6); // variable-width fallback for empty glyph is ADV (6)
}

#[test]
fn print_width_next_column_clear_variable() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let x = 5;
    let y = 20;
    let w = fbm.print_text("im", x, y, 1, false, 1, false);
    // Column at x + w should be clear (spacing accounted for in return width)
    let col = count_drawn_in_col(&mut fbm, x + w, y, 8);
    assert_eq!(col, 0);
}

#[test]
fn print_scale2_width_next_column_clear() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let x = 7;
    let y = 30;
    let w = fbm.print_text("im", x, y, 2, false, 2, false);
    // Scale 2, bounding column at x + w should be empty
    let col = count_drawn_in_col(&mut fbm, x + w, y, 8 * 2);
    assert_eq!(col, 0);
}

#[test]
fn print_newline_advance_scale1() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let x = 12;
    let y = 12;
    let w = fbm.print_text("A\nA", x, y, 1, false, 1, false);
    // First line draws near y..y+7; second starts at y+6
    // Ensure at least one pixel exists in each line's starting row range within [x, x+w)
    let mut found_top = false;
    let mut found_second = false;
    for xx in x..(x + w) {
        if fbm.pix(xx, y, None).unwrap_or(0) != 0 {
            found_top = true;
        }
        if fbm.pix(xx, y + 6, None).unwrap_or(0) != 0 {
            found_second = true;
        }
    }
    assert!(found_top);
    assert!(found_second);
}
