use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::gfx::framebuffer::Framebuffer;

#[allow(dead_code)]
fn count_nonzero(fb: &mut Framebuffer) -> usize {
    let (w, h) = (Framebuffer::WIDTH as i32, Framebuffer::HEIGHT as i32);
    let mut c = 0usize;
    for y in 0..h {
        for x in 0..w {
            if fb.pix(x, y, None).unwrap_or(0) != 0 {
                c += 1;
            }
        }
    }
    c
}

#[test]
fn tri_shared_edge_shallow_slope_tiles_rect() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    // Rectangle region 40x23 at (50,20)
    let x0 = 50;
    let y0 = 20;
    let w = 40;
    let h = 23;
    // Two triangles sharing a shallow edge: (x0,y0)->(x0+w,y0+h)->(x0,y0+h) and (x0,y0)->(x0+w,y0)->(x0+w,y0+h)
    fbm.tri(x0, y0, x0 + w, y0 + h, x0, y0 + h, 2);
    fbm.tri(x0, y0, x0 + w, y0, x0 + w, y0 + h, 2);
    // Count colored pixels inside the rectangle bounds
    let mut area = 0usize;
    for yy in y0..y0 + h {
        for xx in x0..x0 + w {
            if fbm.pix(xx, yy, None).unwrap_or(0) != 0 {
                area += 1;
            }
        }
    }
    assert_eq!(area as i32, (w * h));
}

#[test]
fn ellib_cardinal_points_aspect_wide() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut fbm = fb.borrow_mut();
    fbm.cls(0);
    let cx = 100;
    let cy = 60;
    let a = 20; // x radius
    let b = 5; // y radius
    fbm.ellib(cx, cy, a, b, 3);
    // Cardinals should be lit
    assert!(fbm.pix(cx + a, cy, None).unwrap_or(0) != 0);
    assert!(fbm.pix(cx - a, cy, None).unwrap_or(0) != 0);
    assert!(fbm.pix(cx, cy + b, None).unwrap_or(0) != 0);
    assert!(fbm.pix(cx, cy - b, None).unwrap_or(0) != 0);
    // One pixel outside cardinals should remain background
    assert_eq!(fbm.pix(cx + a + 1, cy, None).unwrap_or(0), 0);
    assert_eq!(fbm.pix(cx - a - 1, cy, None).unwrap_or(0), 0);
    assert_eq!(fbm.pix(cx, cy + b + 1, None).unwrap_or(0), 0);
    assert_eq!(fbm.pix(cx, cy - b - 1, None).unwrap_or(0), 0);
}
