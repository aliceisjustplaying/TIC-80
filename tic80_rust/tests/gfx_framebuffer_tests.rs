use tic80_rust::gfx::framebuffer::{dimensions, Framebuffer};

fn fb_all_pixels_equal(fb: &mut Framebuffer, expected: u8) -> bool {
    let (w, h) = dimensions();
    for y in 0..(h as i32) {
        for x in 0..(w as i32) {
            if fb.pix(x, y, None) != Some(expected & 0x0F) {
                return false;
            }
        }
    }
    true
}

#[test]
fn cls_fills_entire_buffer() {
    let mut fb = Framebuffer::new();
    fb.cls(7);
    assert!(fb_all_pixels_equal(&mut fb, 7));
}

#[test]
fn pix_read_write_and_bounds() {
    let mut fb = Framebuffer::new();
    // in-bounds set/read
    assert!(fb.set_pixel(0, 0, 0xFF)); // should mask to 0x0F
    assert_eq!(fb.pix(0, 0, None), Some(0x0F));

    // out-of-bounds read
    assert_eq!(fb.pix(-1, 0, None), None);
    assert_eq!(fb.pix(0, -1, None), None);
    let (w, h) = dimensions();
    assert_eq!(fb.pix(w as i32, 0, None), None);
    assert_eq!(fb.pix(0, h as i32, None), None);

    // out-of-bounds write via set_pixel reports false
    assert!(!fb.set_pixel(-1, 0, 1));
    assert!(!fb.set_pixel(0, -1, 1));
    assert!(!fb.set_pixel(w as i32, 0, 1));
    assert!(!fb.set_pixel(0, h as i32, 1));
}

#[test]
fn rect_fill_and_clipping() {
    let mut fb = Framebuffer::new();
    fb.cls(0);
    // A rect that partially lies outside should clip
    fb.rect(-5, -3, 10, 8, 9);
    // Count colored pixels; expected area is clipped to [0,w) x [0,h)
    let (w, h) = dimensions();
    let x0 = 0i32.max(-5);
    let y0 = 0i32.max(-3);
    let x1 = ( -5 + 10).min(w as i32);
    let y1 = ( -3 + 8).min(h as i32);
    let expected = (x1 - x0).max(0) as usize * (y1 - y0).max(0) as usize;
    let mut count = 0usize;
    for y in 0..(h as i32) {
        for x in 0..(w as i32) {
            if fb.pix(x, y, None) == Some(9) {
                count += 1;
            }
        }
    }
    assert_eq!(count, expected);

    // Fully out-of-bounds should do nothing
    let mut fb2 = Framebuffer::new();
    fb2.cls(2);
    fb2.rect(-1000, -1000, 10, 10, 5);
    assert!(fb_all_pixels_equal(&mut fb2, 2));
}

fn count_color(fb: &mut Framebuffer, color: u8) -> usize {
    let (w, h) = dimensions();
    let mut c = 0usize;
    for y in 0..(h as i32) {
        for x in 0..(w as i32) {
            if fb.pix(x, y, None) == Some(color & 0x0F) {
                c += 1;
            }
        }
    }
    c
}

#[test]
fn line_basic_counts_and_endpoints() {
    // Horizontal line
    let mut hfb = Framebuffer::new();
    hfb.cls(0);
    hfb.line(0, 0, 10, 0, 3);
    assert_eq!(hfb.pix(0, 0, None), Some(3));
    assert_eq!(hfb.pix(10, 0, None), Some(3));
    assert_eq!(count_color(&mut hfb, 3), 11);

    // Vertical line
    let mut vfb = Framebuffer::new();
    vfb.cls(0);
    vfb.line(0, 0, 0, 7, 5);
    assert_eq!(vfb.pix(0, 0, None), Some(5));
    assert_eq!(vfb.pix(0, 7, None), Some(5));
    assert_eq!(count_color(&mut vfb, 5), 8);

    // Diagonal-ish line and reverse should have same count = max(dx,dy)+1
    let (x0, y0, x1, y1): (i32, i32, i32, i32) = (0, 0, 10, 7);
    let expected = (x1 - x0).abs().max((y1 - y0).abs()) + 1;

    let mut a = Framebuffer::new();
    a.cls(0);
    a.line(x0, y0, x1, y1, 4);
    assert_eq!(a.pix(x0, y0, None), Some(4));
    assert_eq!(a.pix(x1, y1, None), Some(4));
    assert_eq!(count_color(&mut a, 4) as i32, expected);

    let mut b = Framebuffer::new();
    b.cls(0);
    b.line(x1, y1, x0, y0, 4);
    assert_eq!(b.pix(x0, y0, None), Some(4));
    assert_eq!(b.pix(x1, y1, None), Some(4));
    assert_eq!(count_color(&mut b, 4) as i32, expected);
}

#[test]
fn blit_to_rgba_maps_palette() {
    let mut fb = Framebuffer::new();
    fb.cls(0);
    // set three sample pixels to known colors
    fb.set_pixel(0, 0, 0);   // black
    fb.set_pixel(1, 0, 9);   // orange
    fb.set_pixel(2, 0, 15);  // peach

    let (w, h) = dimensions();
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    fb.blit_to_rgba(&mut rgba);

    // Helpers to read RGBA at (x,y)
    let idx = |x: u32, y: u32| -> usize { ((y * w + x) * 4) as usize };

    // Known palette entries from framebuffer.rs
    assert_eq!(&rgba[idx(0, 0)..idx(0, 0) + 4], &[0x00, 0x00, 0x00, 0xFF]);
    assert_eq!(&rgba[idx(1, 0)..idx(1, 0) + 4], &[0xFF, 0xA3, 0x00, 0xFF]);
    assert_eq!(&rgba[idx(2, 0)..idx(2, 0) + 4], &[0xFF, 0xCC, 0xAA, 0xFF]);
}
