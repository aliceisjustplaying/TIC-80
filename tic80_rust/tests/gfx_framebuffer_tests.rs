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

#[allow(dead_code)]
fn fb_hash(fb: &mut Framebuffer) -> u64 {
    // Simple FNV-1a over pixel indices via pix reads
    let (w, h) = dimensions();
    let mut hash: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;
    for y in 0..(h as i32) {
        for x in 0..(w as i32) {
            let b = fb.pix(x, y, None).unwrap_or(0);
            hash ^= b as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
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

#[test]
fn rectb_draws_border() {
    let mut fb = Framebuffer::new();
    fb.cls(0);
    fb.rectb(1, 1, 3, 3, 5);
    // Expected 3x3 border has 8 pixels set
    assert_eq!(count_color(&mut fb, 5), 8);
    // Interior remains background
    assert_eq!(fb.pix(2, 2, None), Some(0));
}

#[test]
fn clip_limits_drawing_and_reset() {
    let mut fb = Framebuffer::new();
    fb.cls(1);
    // Clip to a 1x1 at origin
    fb.clip(0, 0, 1, 1);
    fb.rect(0, 0, 10, 10, 7);
    // Only (0,0) can be changed by rect under this clip
    assert_eq!(fb.pix(0, 0, None), Some(7));
    assert_eq!(fb.pix(1, 0, None), Some(1));
    assert_eq!(fb.pix(0, 1, None), Some(1));

    // Reset clip and draw a rect filling a small area
    fb.clip_reset();
    fb.rect(0, 0, 2, 2, 9);
    assert_eq!(fb.pix(0, 0, None), Some(9));
    assert_eq!(fb.pix(1, 1, None), Some(9));
}

#[test]
fn print_width_fixed_vs_variable_and_newline() {
    let mut fb = Framebuffer::new();
    fb.cls(0);
    // Fixed width: width = len * 6 * scale
    let w1 = fb.print_text("AB", 10, 10, 1, true, 1, false);
    assert_eq!(w1, 2 * 6);
    let w2 = fb.print_text("AB", 10, 10, 1, true, 2, false);
    assert_eq!(w2, 2 * 6 * 2);
    // Variable width should be <= fixed width for same string/scale
    let v1 = fb.print_text("AB", 10, 10, 1, false, 1, false);
    assert!(v1 <= w1);

    // Newlines: expect drawing on initial row and on y+6 (scale=1)
    let mut fb2 = Framebuffer::new();
    fb2.cls(0);
    let _ = fb2.print_text("A\nA", 0, 0, 15, true, 1, false);
    // Something on row 0
    let mut any_row0 = false;
    for x in 0..8 {
        if fb2.pix(x, 0, None) == Some(15) { any_row0 = true; break; }
    }
    assert!(any_row0);
    // And something on row 6
    let mut any_row6 = false;
    for x in 0..8 {
        if fb2.pix(x, 6, None) == Some(15) { any_row6 = true; break; }
    }
    assert!(any_row6);
}

#[test]
fn clip_affects_pix_write() {
    let mut fb = Framebuffer::new();
    fb.cls(2);
    fb.clip(1, 1, 1, 1); // only (1,1)
    // Write outside clip
    let _ = fb.pix(0, 0, Some(7));
    // Write inside clip
    let _ = fb.pix(1, 1, Some(7));
    // Validate
    assert_eq!(fb.pix(0, 0, None), Some(2));
    assert_eq!(fb.pix(1, 1, None), Some(7));
}

#[test]
fn robust_oob_line_and_rectb() {
    let mut fb = Framebuffer::new();
    fb.cls(0);
    // Very long line across/outside bounds
    fb.line(-100, -100, 1000, 2000, 9);
    // Should produce some in-bounds pixels
    assert!(count_color(&mut fb, 9) > 0);

    // Rect border with negative origin that crosses viewport
    fb.rectb(-5, -5, 12, 12, 4);
    // Perimeter segments that lie in-bounds should be colored
    assert_eq!(fb.pix(6, 0, None), Some(4)); // right edge
    assert_eq!(fb.pix(0, 6, None), Some(4)); // bottom edge
}

#[test]
fn circb_cardinals_and_oob() {
    let mut fb = Framebuffer::new();
    fb.cls(0);
    let cx = 20;
    let cy = 20;
    let r = 5;
    fb.circb(cx, cy, r, 7);
    // cardinal points
    assert_eq!(fb.pix(cx + r, cy, None), Some(7));
    assert_eq!(fb.pix(cx - r, cy, None), Some(7));
    assert_eq!(fb.pix(cx, cy + r, None), Some(7));
    assert_eq!(fb.pix(cx, cy - r, None), Some(7));
    // just outside should remain background
    assert_eq!(fb.pix(cx + r + 1, cy, None), Some(0));
}

#[test]
fn circ_fill_center_row_and_clip() {
    let mut fb = Framebuffer::new();
    fb.cls(0);
    let cx = 30;
    let cy = 30;
    let r = 4;
    fb.circ(cx, cy, r, 5);
    // center row should be filled from cx-r .. cx+r
    for x in (cx - r)..=(cx + r) {
        assert_eq!(fb.pix(x, cy, None), Some(5));
    }
    assert_eq!(fb.pix(cx - r - 1, cy, None), Some(0));
    assert_eq!(fb.pix(cx + r + 1, cy, None), Some(0));

    // clipping restricts drawing to 1x1
    let mut fb2 = Framebuffer::new();
    fb2.cls(2);
    fb2.clip(0, 0, 1, 1);
    fb2.circ(0, 0, 5, 9);
    assert_eq!(fb2.pix(0, 0, None), Some(9));
    assert_eq!(fb2.pix(1, 0, None), Some(2));
    assert_eq!(fb2.pix(0, 1, None), Some(2));
}

#[test]
fn circ_zero_radius_draws_center() {
    let mut fb = Framebuffer::new();
    fb.cls(0);
    fb.circ(10, 10, 0, 3);
    assert_eq!(fb.pix(10, 10, None), Some(3));
    let mut fb2 = Framebuffer::new();
    fb2.cls(0);
    fb2.circb(10, 10, 0, 4);
    assert_eq!(fb2.pix(10, 10, None), Some(4));
}
