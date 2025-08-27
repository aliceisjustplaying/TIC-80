#![allow(clippy::cast_possible_truncation)]
use crate::gfx::framebuffer::Framebuffer;
use std::cell::RefCell;
use std::rc::Rc;

/// Total RAM size (bytes) exposed to peek/poke APIs.
const RAM_TOTAL: usize = 96 * 1024; // 96KB
/// VRAM window size (bytes). For the prototype, the first 16KB mirrors TIC-80's VRAM.
const VRAM_SIZE: usize = 16 * 1024;
/// Screen dimensions (copied from framebuffer constants for clarity).
const SCREEN_WIDTH: usize = crate::gfx::framebuffer::Framebuffer::WIDTH as usize;
const SCREEN_HEIGHT: usize = crate::gfx::framebuffer::Framebuffer::HEIGHT as usize;
/// Total number of screen pixels.
const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT; // 240*136 = 32640
/// Pixels are nibble-packed into VRAM screen bytes: 2 pixels per byte (lo = even, hi = odd).
const PIXELS_PER_BYTE: usize = 2;
/// Number of bytes in VRAM dedicated to the screen nibble pairs.
const VRAM_SCREEN_BYTES: usize = SCREEN_PIXELS / PIXELS_PER_BYTE; // 16320 (0x3FC0)

pub struct Memory {
    ram: Vec<u8>,
    fb: Rc<RefCell<Framebuffer>>, // for VRAM screen mapping
}

impl Memory {
    #[must_use]
    pub fn new(fb: Rc<RefCell<Framebuffer>>) -> Self {
        Self {
            ram: vec![0; RAM_TOTAL],
            fb,
        }
    }

    // Bit masks for sub-byte operations
    const NIBBLE_MASK: u8 = 0x0F; // 4 bits
    const TWO_BIT_MASK: u8 = 0x03; // 2 bits
    const ONE_BIT_MASK: u8 = 0x01; // 1 bit

    // 8-bit read/write with VRAM screen mapping
    fn get_byte(&self, addr: usize) -> u8 {
        if addr < VRAM_SCREEN_BYTES {
            // Pack 2 pixels from framebuffer into one byte (low nibble = even pixel)
            let p = addr * 2; // pixel index
            let mut fb = self.fb.borrow_mut();
            let (w, h) = (SCREEN_WIDTH, SCREEN_HEIGHT);
            // Framebuffer stores 1 byte per pixel index; map linear order row-major
            // pixel p is (x=p%w, y=p/w)
            #[allow(clippy::cast_possible_truncation)]
            let mut get_px = |pi: usize| -> u8 {
                if pi < w * h {
                    fb.pix((pi % w) as i32, (pi / w) as i32, None).unwrap_or(0) & Self::NIBBLE_MASK
                } else {
                    0
                }
            };
            let lo = get_px(p);
            let hi = get_px(p + 1);
            (lo & Self::NIBBLE_MASK) | ((hi & Self::NIBBLE_MASK) << 4)
        } else if addr < VRAM_SIZE {
            // other VRAM bytes (palette, etc.): just return RAM view for now
            self.ram[addr]
        } else if addr < RAM_TOTAL {
            self.ram[addr]
        } else {
            0
        }
    }

    fn set_byte(&mut self, addr: usize, val: u8) {
        if addr < VRAM_SCREEN_BYTES {
            // Unpack byte to 2 pixels in the framebuffer
            let p = addr * 2;
            let lo = val & Self::NIBBLE_MASK;
            let hi = (val >> 4) & Self::NIBBLE_MASK;
            let mut fbm = self.fb.borrow_mut();
            let w = SCREEN_WIDTH;
            #[allow(clippy::cast_possible_truncation)]
            let set_px = |fb: &mut Framebuffer, pi: usize, v: u8| {
                let x = (pi % w) as i32;
                let y = (pi / w) as i32;
                let _ = fb.set_pixel_unclipped(x, y, v);
            };
            set_px(&mut fbm, p, lo);
            set_px(&mut fbm, p + 1, hi);
            // keep RAM mirror in case code inspects it directly
            self.ram[addr] = val;
        } else if addr < RAM_TOTAL {
            self.ram[addr] = val;
            // TODO: if palette region is written, we may later plumb it to framebuffer palette
        }
    }

    #[must_use]
    pub fn peek(&self, addr: usize) -> u8 {
        self.get_byte(addr)
    }
    pub fn poke(&mut self, addr: usize, val: u8) {
        self.set_byte(addr, val);
    }

    // bit-packed peeks/pokes across entire 96KB (VRAM included)
    #[must_use]
    pub fn peek_bits(&self, addr: usize, bits: u8) -> u8 {
        match bits {
            8 => self.peek(addr),
            4 => {
                let byte = self.peek(addr >> 1);
                if (addr & 1) == 0 {
                    byte & Self::NIBBLE_MASK
                } else {
                    (byte >> 4) & Self::NIBBLE_MASK
                }
            }
            2 => {
                let byte = self.peek(addr >> 2);
                let shift = (addr & 0b11) * 2;
                (byte >> shift) & Self::TWO_BIT_MASK
            }
            1 => {
                let byte = self.peek(addr >> 3);
                let shift = addr & 0b111;
                (byte >> shift) & Self::ONE_BIT_MASK
            }
            _ => 0,
        }
    }

    pub fn poke_bits(&mut self, addr: usize, bits: u8, val: u8) {
        match bits {
            8 => self.poke(addr, val),
            4 => {
                let mut byte = self.peek(addr >> 1);
                if (addr & 1) == 0 {
                    byte = (byte & 0xF0) | (val & Self::NIBBLE_MASK);
                } else {
                    byte = (byte & Self::NIBBLE_MASK) | ((val & Self::NIBBLE_MASK) << 4);
                }
                self.poke(addr >> 1, byte);
            }
            2 => {
                let mut byte = self.peek(addr >> 2);
                let shift = (addr & 0b11) * 2;
                let mask = !(Self::TWO_BIT_MASK << shift);
                byte = (byte & mask) | ((val & Self::TWO_BIT_MASK) << shift);
                self.poke(addr >> 2, byte);
            }
            1 => {
                let mut byte = self.peek(addr >> 3);
                let shift = addr & 0b111;
                let mask = !(Self::ONE_BIT_MASK << shift);
                byte = (byte & mask) | ((val & Self::ONE_BIT_MASK) << shift);
                self.poke(addr >> 3, byte);
            }
            _ => {}
        }
    }

    pub fn memcpy(&mut self, dst: usize, src: usize, size: usize) {
        if size == 0 {
            return;
        }
        // Handle overlap with correct direction
        if src < dst && src + size > dst {
            for i in (0..size).rev() {
                let b = self.get_byte(src + i);
                self.set_byte(dst + i, b);
            }
        } else {
            for i in 0..size {
                let b = self.get_byte(src + i);
                self.set_byte(dst + i, b);
            }
        }
    }

    pub fn memset(&mut self, dst: usize, value: u8, size: usize) {
        for i in 0..size {
            self.set_byte(dst + i, value);
        }
    }
}
