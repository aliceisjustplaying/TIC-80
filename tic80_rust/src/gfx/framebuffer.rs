use std::sync::OnceLock;

// Default 16-color TIC-80 palette (sRGB) as RGBA8
const PALETTE: [(u8, u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00, 0xFF),
    (0x1D, 0x2B, 0x53, 0xFF),
    (0x7E, 0x25, 0x53, 0xFF),
    (0x00, 0x87, 0x51, 0xFF),
    (0xAB, 0x52, 0x36, 0xFF),
    (0x5F, 0x57, 0x4F, 0xFF),
    (0xC2, 0xC3, 0xC7, 0xFF),
    (0xFF, 0xF1, 0xE8, 0xFF),
    (0xFF, 0x00, 0x4D, 0xFF),
    (0xFF, 0xA3, 0x00, 0xFF),
    (0xFF, 0xEC, 0x27, 0xFF),
    (0x00, 0xE4, 0x36, 0xFF),
    (0x29, 0xAD, 0xFF, 0xFF),
    (0x83, 0x76, 0x9C, 0xFF),
    (0xFF, 0x77, 0xA8, 0xFF),
    (0xFF, 0xCC, 0xAA, 0xFF),
];

const WIDTH: u32 = 240;
const HEIGHT: u32 = 136;

// TIC-80 default font bytes as 8 rows per glyph, 1 bit per pixel (LSB is x=0)
static FONT_TEXT: &str = include_str!("../../../src/core/font.inl");
static FONT_BYTES: OnceLock<Vec<u8>> = OnceLock::new();

fn font_bytes() -> &'static [u8] {
    FONT_BYTES.get_or_init(|| {
        // Parse C-style hex/decimal list into bytes
        let mut out = Vec::with_capacity(1024);
        for tok in FONT_TEXT.split(|c: char| c.is_whitespace() || c == ',') {
            if tok.is_empty() {
                continue;
            }
            let t = tok.trim();
            let val = if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                u8::from_str_radix(hex, 16).ok()
            } else {
                t.parse::<u8>().ok()
            };
            if let Some(b) = val {
                out.push(b);
            }
        }
        out
    })
}

pub struct Framebuffer {
    // 240x136 palette indices (0..15)
    idx: Vec<u8>,
}

impl Framebuffer {
    pub fn new() -> Self {
        Self {
            idx: vec![0; (WIDTH * HEIGHT) as usize],
        }
    }

    // cls(color): fill framebuffer with palette index
    pub fn cls(&mut self, color: u8) {
        self.idx.fill(color & 0x0F);
    }

    // pix(x,y[,color]): if Some(color) -> write; else -> read
    pub fn pix(&mut self, x: i32, y: i32, color: Option<u8>) -> Option<u8> {
        if x < 0 || y < 0 || x as u32 >= WIDTH || y as u32 >= HEIGHT {
            return None;
        }
        let i = (y as u32 * WIDTH + x as u32) as usize;
        match color {
            Some(c) => {
                self.idx[i] = c & 0x0F;
                None
            }
            None => Some(self.idx[i] & 0x0F),
        }
    }

    // Blit to RGBA buffer for pixels
    pub fn blit_to_rgba(&self, rgba: &mut [u8]) {
        for (i, idx) in self.idx.iter().copied().enumerate() {
            let (r, g, b, a) = PALETTE[(idx & 0x0F) as usize];
            let o = i * 4;
            rgba[o] = r;
            rgba[o + 1] = g;
            rgba[o + 2] = b;
            rgba[o + 3] = a;
        }
    }

    // Draw a line using integer Bresenham
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u8) {
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let c = color & 0x0F;
        loop {
            let _ = self.pix(x0, y0, Some(c));
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    // Filled rectangle
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        if w <= 0 || h <= 0 {
            return;
        }
        let c = color & 0x0F;
        for yy in y..y + h {
            for xx in x..x + w {
                let _ = self.pix(xx, yy, Some(c));
            }
        }
    }

    // Print text using TIC-80 default font (5x8 glyphs, 1px spacing)
    #[allow(clippy::too_many_arguments)]
    pub fn print_text(
        &mut self,
        text: &str,
        x: i32,
        mut y: i32,
        color: u8,
        fixed: bool,
        scale: i32,
        _small: bool,
    ) -> i32 {
        // Match TIC-80 print/drawText behavior
        const GLYPH_W: usize = 8;
        const GLYPH_H: usize = 8;
        const ADV: i32 = 6; // TIC_FONT_WIDTH
        if scale <= 0 {
            return 0;
        }
        let cidx = color & 0x0F;
        let font = font_bytes();

        let mut pos = x;
        let mut max_pos = x;

        for ch in text.chars() {
            if ch == '\n' {
                if pos > max_pos {
                    max_pos = pos;
                }
                pos = x;
                y += ADV * scale; // TIC_FONT_HEIGHT (6)
                continue;
            }

            let code = (ch as u32 & 0x7F) as usize;
            let base = code * GLYPH_H;
            if base + GLYPH_H > font.len() {
                pos += ADV * scale;
                continue;
            }

            let (start_col, width_cols) = if !fixed {
                let mut left = GLYPH_W;
                let mut right = 0;
                for row in 0..GLYPH_H {
                    let mask = font[base + row];
                    if mask != 0 {
                        let mut l = 0;
                        while l < GLYPH_W && ((mask >> l) & 1) == 0 {
                            l += 1;
                        }
                        let mut r = GLYPH_W;
                        while r > 0 && ((mask >> (r - 1)) & 1) == 0 {
                            r -= 1;
                        }
                        if l < left {
                            left = l;
                        }
                        if r > right {
                            right = r;
                        }
                    }
                }
                let width = right.saturating_sub(left);
                (left, width)
            } else {
                (0, GLYPH_W)
            };

            // Draw glyph
            for row in 0..GLYPH_H {
                let mask = font[base + row];
                for col in 0..width_cols {
                    let bit_idx = start_col + col;
                    if bit_idx < GLYPH_W && ((mask >> bit_idx) & 1) != 0 {
                        let px = pos + (col as i32) * scale;
                        let py = y + (row as i32) * scale;
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let _ = self.pix(px + sx, py + sy, Some(cidx));
                            }
                        }
                    }
                }
            }

            // Advance
            if !fixed {
                if width_cols > 0 {
                    pos += ((width_cols as i32) + 1) * scale;
                } else {
                    pos += ADV * scale;
                }
            } else {
                pos += ADV * scale;
            }
        }

        if pos > max_pos {
            pos - x
        } else {
            max_pos - x
        }
    }
}

pub fn dimensions() -> (u32, u32) {
    (WIDTH, HEIGHT)
}
