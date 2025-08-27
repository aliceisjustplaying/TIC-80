use crate::gfx::framebuffer::Framebuffer;
use std::cell::RefCell;
use std::rc::Rc;

const RAM_TOTAL: usize = 96 * 1024; // 96KB
const VRAM_SIZE: usize = 16 * 1024; // first 16KB of RAM is VRAM window
const VRAM_SCREEN_BYTES: usize = 0x3FC0; // 16320 bytes of screen nibble pairs

pub struct Memory {
    ram: Vec<u8>,
    fb: Rc<RefCell<Framebuffer>>, // for VRAM screen mapping
}

impl Memory {
    pub fn new(fb: Rc<RefCell<Framebuffer>>) -> Self {
        Self {
            ram: vec![0; RAM_TOTAL],
            fb,
        }
    }

    // 8-bit read/write with VRAM screen mapping
    fn get_byte(&self, addr: usize) -> u8 {
        if addr < VRAM_SCREEN_BYTES {
            // pack 2 pixels from framebuffer into one byte (low nibble = even pixel)
            let p = addr * 2; // pixel index
            let mut fb = self.fb.borrow_mut();
            let (w, h) = (Framebuffer::WIDTH as usize, Framebuffer::HEIGHT as usize);
            // Framebuffer stores 1 byte per pixel index; map linear order row-major
            // pixel p is (x=p%w, y=p/w)
            let mut get_px = |pi: usize| -> u8 {
                if pi < w * h {
                    fb.pix((pi % w) as i32, (pi / w) as i32, None).unwrap_or(0) & 0x0F
                } else {
                    0
                }
            };
            let lo = get_px(p);
            let hi = get_px(p + 1);
            (lo & 0x0F) | ((hi & 0x0F) << 4)
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
            // unpack to 2 pixels
            let p = addr * 2;
            let lo = val & 0x0F;
            let hi = (val >> 4) & 0x0F;
            let mut fbm = self.fb.borrow_mut();
            let w = Framebuffer::WIDTH as usize;
            let set_px = |fb: &mut Framebuffer, pi: usize, v: u8| {
                let x = (pi % w) as i32;
                let y = (pi / w) as i32;
                let _ = fb.pix(x, y, Some(v));
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

    pub fn peek(&self, addr: usize) -> u8 {
        self.get_byte(addr)
    }
    pub fn poke(&mut self, addr: usize, val: u8) {
        self.set_byte(addr, val);
    }

    // bit-packed peeks/pokes across entire 96KB (VRAM included)
    pub fn peek_bits(&self, addr: usize, bits: u8) -> u8 {
        match bits {
            8 => self.peek(addr),
            4 => {
                let byte = self.peek(addr >> 1);
                if (addr & 1) == 0 {
                    byte & 0x0F
                } else {
                    (byte >> 4) & 0x0F
                }
            }
            2 => {
                let byte = self.peek(addr >> 2);
                let shift = (addr & 0b11) * 2;
                (byte >> shift) & 0x03
            }
            1 => {
                let byte = self.peek(addr >> 3);
                let shift = addr & 0b111;
                (byte >> shift) & 0x01
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
                    byte = (byte & 0xF0) | (val & 0x0F);
                } else {
                    byte = (byte & 0x0F) | ((val & 0x0F) << 4);
                }
                self.poke(addr >> 1, byte);
            }
            2 => {
                let mut byte = self.peek(addr >> 2);
                let shift = (addr & 0b11) * 2;
                let mask = !(0x03u8 << shift);
                byte = (byte & mask) | ((val & 0x03) << shift);
                self.poke(addr >> 2, byte);
            }
            1 => {
                let mut byte = self.peek(addr >> 3);
                let shift = addr & 0b111;
                let mask = !(1u8 << shift);
                byte = (byte & mask) | ((val & 0x01) << shift);
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
