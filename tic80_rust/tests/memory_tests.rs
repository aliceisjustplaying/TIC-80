use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::core::memory::Memory;
use tic80_rust::gfx::framebuffer::Framebuffer;

#[test]
fn poke4_sets_framebuffer_pixel() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb.clone());
    // Address 0 corresponds to first two pixels (0,0) and (1,0)
    mem.poke_bits(0, 4, 0xA);
    assert_eq!(fb.borrow_mut().pix(0, 0, None), Some(0xA));
    // High nibble is pixel 1
    mem.poke_bits(1, 4, 0x3);
    assert_eq!(fb.borrow_mut().pix(1, 0, None), Some(0x3));
}

#[test]
fn peek4_reads_back_nibble() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mem = Memory::new(fb.clone());
    // Set two pixels via framebuffer then read nibbles
    fb.borrow_mut().pix(0, 0, Some(7));
    fb.borrow_mut().pix(1, 0, Some(12));
    assert_eq!(mem.peek_bits(0, 4), 7);
    assert_eq!(mem.peek_bits(1, 4), 12 & 0x0F);
}

#[test]
fn memcpy_and_memset_affect_vram() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb.clone());
    // memset first 10 bytes of VRAM screen -> sets first 20 pixels (pairs) to (0x5,0x5)
    mem.memset(0, 0x55, 10);
    for x in 0..20 {
        assert_eq!(fb.borrow_mut().pix(x, 0, None), Some(0x5));
    }

    // prepare source bytes with pattern 0xAB -> (0xB,0xA) on pixels
    let src = 40000usize; // within RAM region beyond VRAM
    for i in 0..4 {
        mem.poke(src + i, 0xAB);
    }
    mem.memcpy(0, src, 4); // copy into beginning of VRAM
                           // First 8 pixels now map from 0xAB pairs
    let mut px = vec![];
    for x in 0..8 {
        px.push(fb.borrow_mut().pix(x, 0, None).unwrap());
    }
    assert_eq!(&px, &[0xB, 0xA, 0xB, 0xA, 0xB, 0xA, 0xB, 0xA]);
}

#[test]
fn peek_poke_bits_general_ram() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb);
    let base = 90000usize; // inside 96KB
    mem.memset(base, 0x00, 2);
    mem.poke_bits(base * 2, 4, 0xF); // address in nibbles
    assert_eq!(mem.peek(base), 0x0F);
    mem.poke_bits(base * 8 + 7, 1, 1); // set MSB of first byte
    assert_eq!(mem.peek(base), 0x8F);
}

#[test]
fn vram_writes_ignore_clip() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb.clone());
    // Set a clip that excludes the first pixel (0,0)
    fb.borrow_mut().clip(10, 10, 10, 10);
    // Write to VRAM screen first byte low nibble -> pixel (0,0)
    mem.poke_bits(0, 4, 0xC);
    // Despite clip, the pixel must update
    assert_eq!(fb.borrow_mut().pix(0, 0, None), Some(0xC));
}
