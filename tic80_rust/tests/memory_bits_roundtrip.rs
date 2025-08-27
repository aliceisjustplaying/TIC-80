use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::core::memory::Memory;
use tic80_rust::gfx::framebuffer::Framebuffer;

fn mask_for_bits(bits: u8) -> u8 {
    match bits {
        1 => 0x01,
        2 => 0x03,
        4 => 0x0F,
        8 => 0xFF,
        _ => 0,
    }
}

#[test]
fn roundtrip_peek_poke_bits_general_ram() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb);
    let base = 80_000usize; // within 96KB RAM (beyond VRAM)
                            // exercise a range of addresses for each bit-width with a deterministic pattern
    for &bits in &[1u8, 2, 4, 8] {
        let m = mask_for_bits(bits);
        // ensure a clean slate
        mem.memset(base, 0x00, 128);
        for i in 0..512usize {
            let addr = match bits {
                8 => base + i,
                4 => base * 2 + i, // address in nibbles
                2 => base * 4 + i, // address in 2-bit units
                1 => base * 8 + i, // address in bits
                _ => unreachable!(),
            };
            let val = ((i as u8).wrapping_mul(37)) & m;
            mem.poke_bits(addr, bits, val);
            assert_eq!(mem.peek_bits(addr, bits), val);
        }
    }
}

#[test]
fn vram_screen_boundary_write_does_not_bleed() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb.clone());
    // Compute VRAM screen byte count from framebuffer size (2 pixels per byte)
    let w = Framebuffer::WIDTH as usize;
    let h = Framebuffer::HEIGHT as usize;
    let screen_bytes = (w * h) / 2;

    // Set last byte in screen region to a known pattern
    let last_byte_index = screen_bytes - 1;
    mem.poke(last_byte_index, 0xAB);
    // Verify the last two pixels updated accordingly: low nibble -> even pixel (x=w-2), high -> odd (x=w-1)
    let px0 = fb
        .borrow_mut()
        .pix((w - 1) as i32, (h - 1) as i32, None)
        .unwrap();
    let px1 = fb
        .borrow_mut()
        .pix((w - 2) as i32, (h - 1) as i32, None)
        .unwrap();
    assert_eq!(px0, 0x0A); // last pixel (odd index) gets high nibble
    assert_eq!(px1, 0x0B); // previous pixel (even index) gets low nibble

    // Now write the very next VRAM byte (first non-screen VRAM byte)
    let before0 = fb
        .borrow_mut()
        .pix((w - 1) as i32, (h - 1) as i32, None)
        .unwrap();
    let before1 = fb
        .borrow_mut()
        .pix((w - 2) as i32, (h - 1) as i32, None)
        .unwrap();
    mem.poke(screen_bytes, 0xFF);
    // Framebuffer should not change for non-screen VRAM writes
    let after0 = fb
        .borrow_mut()
        .pix((w - 1) as i32, (h - 1) as i32, None)
        .unwrap();
    let after1 = fb
        .borrow_mut()
        .pix((w - 2) as i32, (h - 1) as i32, None)
        .unwrap();
    assert_eq!(before0, after0);
    assert_eq!(before1, after1);
}
