use std::cell::RefCell;
use std::rc::Rc;

use tic80_rust::core::memory::Memory;
use tic80_rust::gfx::framebuffer::Framebuffer;

#[test]
fn two_bit_cross_byte_alignment() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb);
    let base = 70_000usize; // general RAM region
    mem.memset(base, 0x00, 2);
    // Write last 2 bits (shift=6) of first byte via 2-bit addressing
    // addr = byte_idx*4 + slot, slot in [0..3]
    let a0 = base * 4 + 3;
    mem.poke_bits(a0, 2, 0b01);
    assert_eq!(mem.peek(base), 0b01 << 6);
    // Write first 2 bits (shift=0) of next byte via 2-bit addressing
    let a1 = (base + 1) * 4;
    mem.poke_bits(a1, 2, 0b10);
    assert_eq!(mem.peek(base + 1), 0b10);
    // Confirm first byte unchanged by second write
    assert_eq!(mem.peek(base), 0b01 << 6);
}

#[test]
fn four_bit_unaligned_nibbles() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb);
    let base = 65_000usize; // general RAM region
    mem.memset(base, 0x00, 3);
    // Write high nibble of first byte (odd nibble address)
    let a_hi = base * 2 + 1;
    mem.poke_bits(a_hi, 4, 0xA);
    assert_eq!(mem.peek(base), 0xA0);
    // Write low nibble of next byte (even nibble address for next byte)
    let a_lo_next = (base + 1) * 2;
    mem.poke_bits(a_lo_next, 4, 0x5);
    assert_eq!(mem.peek(base + 1) & 0x0F, 0x05);
    // Ensure first byte unchanged
    assert_eq!(mem.peek(base), 0xA0);

    // Now write low then high nibble within the same byte
    mem.memset(base + 2, 0x00, 1);
    let a_lo = (base + 2) * 2;
    let a_hi_same = (base + 2) * 2 + 1;
    mem.poke_bits(a_lo, 4, 0x3);
    mem.poke_bits(a_hi_same, 4, 0xC);
    assert_eq!(mem.peek(base + 2), 0xC3);
}

#[test]
fn one_bit_cross_byte_alignment_edges() {
    let fb = Rc::new(RefCell::new(Framebuffer::new()));
    let mut mem = Memory::new(fb);
    let base = 75_000usize; // general RAM region
    mem.memset(base, 0x00, 2);
    // Set MSB of first byte
    let a0 = base * 8 + 7;
    mem.poke_bits(a0, 1, 1);
    assert_eq!(mem.peek(base), 0b1000_0000);
    // Set LSB of next byte
    let a1 = (base + 1) * 8;
    mem.poke_bits(a1, 1, 1);
    assert_eq!(mem.peek(base + 1), 0b0000_0001);
    // Ensure first byte unchanged by second write
    assert_eq!(mem.peek(base), 0b1000_0000);
}
