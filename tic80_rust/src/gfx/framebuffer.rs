#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
use std::sync::OnceLock;

// Default 16-color TIC-80 palette (sRGB) as RGBA8
const COLOR_MASK: u8 = 0x0F;
const PALETTE: [[u8; 4]; 16] = [
    [0x00, 0x00, 0x00, 0xFF],
    [0x1D, 0x2B, 0x53, 0xFF],
    [0x7E, 0x25, 0x53, 0xFF],
    [0x00, 0x87, 0x51, 0xFF],
    [0xAB, 0x52, 0x36, 0xFF],
    [0x5F, 0x57, 0x4F, 0xFF],
    [0xC2, 0xC3, 0xC7, 0xFF],
    [0xFF, 0xF1, 0xE8, 0xFF],
    [0xFF, 0x00, 0x4D, 0xFF],
    [0xFF, 0xA3, 0x00, 0xFF],
    [0xFF, 0xEC, 0x27, 0xFF],
    [0x00, 0xE4, 0x36, 0xFF],
    [0x29, 0xAD, 0xFF, 0xFF],
    [0x83, 0x76, 0x9C, 0xFF],
    [0xFF, 0x77, 0xA8, 0xFF],
    [0xFF, 0xCC, 0xAA, 0xFF],
];

const WIDTH: u32 = 240;
const HEIGHT: u32 = 136;

// TIC-80 default font bytes as 8 rows per glyph, 1 bit per pixel.
// Interpret bits LSB→MSB as columns left→right (bit0 is x=0).
static FONT_TEXT: &str = include_str!("../../assets/fonts/default_font.inl");
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
            let val = t
                .strip_prefix("0x")
                .or_else(|| t.strip_prefix("0X"))
                .map_or_else(
                    || t.parse::<u8>().ok(),
                    |hex| u8::from_str_radix(hex, 16).ok(),
                );
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
    // Clipping rectangle in inclusive-exclusive coords [x0,x1), [y0,y1)
    clip_x0: i32,
    clip_y0: i32,
    clip_x1: i32,
    clip_y1: i32,
}

impl Framebuffer {
    pub const WIDTH: u32 = WIDTH;
    pub const HEIGHT: u32 = HEIGHT;

    #[must_use]
    pub fn new() -> Self {
        Self {
            idx: vec![0; (WIDTH * HEIGHT) as usize],
            clip_x0: 0,
            clip_y0: 0,
            clip_x1: WIDTH as i32,
            clip_y1: HEIGHT as i32,
        }
    }

    // cls(color): fill framebuffer with palette index
    pub fn cls(&mut self, color: u8) {
        self.idx.fill(color & COLOR_MASK);
    }

    // pix(x,y[,color]): if Some(color) -> write; else -> read
    pub fn pix(&mut self, x: i32, y: i32, color: Option<u8>) -> Option<u8> {
        match color {
            Some(c) => {
                let _ = self.set_pixel(x, y, c);
                None
            }
            None => {
                if x < 0 || y < 0 || x as u32 >= WIDTH || y as u32 >= HEIGHT {
                    None
                } else {
                    let i = (y as u32 * WIDTH + x as u32) as usize;
                    Some(self.idx[i] & COLOR_MASK)
                }
            }
        }
    }

    // Write a single pixel; returns true if in-bounds and written
    pub fn set_pixel(&mut self, x: i32, y: i32, color: u8) -> bool {
        if x < self.clip_x0
            || y < self.clip_y0
            || x >= self.clip_x1
            || y >= self.clip_y1
            || x < 0
            || y < 0
            || x as u32 >= WIDTH
            || y as u32 >= HEIGHT
        {
            return false;
        }
        let i = (y as u32 * WIDTH + x as u32) as usize;
        self.idx[i] = color & COLOR_MASK;
        true
    }

    // Unclipped pixel write for memory-to-VRAM mapping; only bounds-checked.
    pub fn set_pixel_unclipped(&mut self, x: i32, y: i32, color: u8) -> bool {
        if x < 0 || y < 0 || x as u32 >= WIDTH || y as u32 >= HEIGHT {
            return false;
        }
        let i = (y as u32 * WIDTH + x as u32) as usize;
        self.idx[i] = color & COLOR_MASK;
        true
    }

    // Set clipping rectangle to intersection with framebuffer and provided rect
    pub fn clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if w <= 0 || h <= 0 {
            self.clip_x0 = 0;
            self.clip_y0 = 0;
            self.clip_x1 = 0;
            self.clip_y1 = 0;
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(WIDTH as i32);
        let y1 = (y + h).min(HEIGHT as i32);
        self.clip_x0 = x0.max(0);
        self.clip_y0 = y0.max(0);
        self.clip_x1 = x1.max(self.clip_x0);
        self.clip_y1 = y1.max(self.clip_y0);
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn clip_reset(&mut self) {
        self.clip_x0 = 0;
        self.clip_y0 = 0;
        self.clip_x1 = WIDTH as i32;
        self.clip_y1 = HEIGHT as i32;
    }

    // Blit to RGBA buffer for pixels
    pub fn blit_to_rgba(&self, rgba: &mut [u8]) {
        for (px, idx) in rgba.chunks_exact_mut(4).zip(self.idx.iter().copied()) {
            let pal = &PALETTE[(idx & COLOR_MASK) as usize];
            px.copy_from_slice(pal);
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
        let c = color & COLOR_MASK;
        loop {
            let _ = self.set_pixel(x0, y0, c);
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

    // Filled rectangle (clipped to framebuffer bounds)
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        if w <= 0 || h <= 0 {
            return;
        }
        let c = color & COLOR_MASK;
        let x0 = x.max(self.clip_x0).max(0);
        let y0 = y.max(self.clip_y0).max(0);
        let x1 = (x + w).min(self.clip_x1).min(WIDTH as i32);
        let y1 = (y + h).min(self.clip_y1).min(HEIGHT as i32);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let width = WIDTH as usize;
        for yy in y0..y1 {
            let base = (yy as usize) * width;
            let start = base + x0 as usize;
            let end = base + x1 as usize;
            self.idx[start..end].fill(c);
        }
    }

    // Rectangle border (one-pixel thick), obeys clipping
    pub fn rectb(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        if w <= 0 || h <= 0 {
            return;
        }
        let c = color & COLOR_MASK;
        let x0 = x;
        let y0 = y;
        let x1 = x + w - 1;
        let y1 = y + h - 1;
        // Top and bottom edges
        for xx in x0..=x1 {
            let _ = self.set_pixel(xx, y0, c);
            let _ = self.set_pixel(xx, y1, c);
        }
        // Left and right edges
        for yy in y0..=y1 {
            let _ = self.set_pixel(x0, yy, c);
            let _ = self.set_pixel(x1, yy, c);
        }
    }

    // Circle border using 8-way symmetry (integer midpoint algorithm)
    pub fn circb(&mut self, cx: i32, cy: i32, r: i32, color: u8) {
        if r < 0 {
            return;
        }
        let c = color & COLOR_MASK;
        if r == 0 {
            let _ = self.set_pixel(cx, cy, c);
            return;
        }
        let mut x = r;
        let mut y = 0;
        let mut err = 1 - r;
        while x >= y {
            // 8 symmetric points
            let _ = self.set_pixel(cx + x, cy + y, c);
            let _ = self.set_pixel(cx + y, cy + x, c);
            let _ = self.set_pixel(cx - y, cy + x, c);
            let _ = self.set_pixel(cx - x, cy + y, c);
            let _ = self.set_pixel(cx - x, cy - y, c);
            let _ = self.set_pixel(cx - y, cy - x, c);
            let _ = self.set_pixel(cx + y, cy - x, c);
            let _ = self.set_pixel(cx + x, cy - y, c);

            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    // Filled circle via horizontal spans using symmetry
    pub fn circ(&mut self, cx: i32, cy: i32, r: i32, color: u8) {
        if r < 0 {
            return;
        }
        let c = color & COLOR_MASK;
        if r == 0 {
            let _ = self.set_pixel(cx, cy, c);
            return;
        }
        let mut x = r;
        let mut y = 0;
        let mut err = 1 - r;
        while x >= y {
            // Draw horizontal spans for the current y and x offsets
            self.hspan(cx - x, cx + x, cy + y, c);
            self.hspan(cx - x, cx + x, cy - y, c);
            self.hspan(cx - y, cx + y, cy + x, c);
            self.hspan(cx - y, cx + y, cy - x, c);

            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    fn hspan(&mut self, x0: i32, x1: i32, y: i32, color: u8) {
        if y < 0 || y as u32 >= HEIGHT {
            return;
        }
        let start = x0.min(x1);
        let end = x0.max(x1);
        for x in start..=end {
            let _ = self.set_pixel(x, y, color);
        }
    }

    // Ellipse border using midpoint algorithm
    #[allow(clippy::cast_possible_truncation)]
    pub fn ellib(&mut self, cx: i32, cy: i32, a: i32, b: i32, color: u8) {
        if a < 0 || b < 0 {
            return;
        }
        let c = color & COLOR_MASK;
        if a == 0 && b == 0 {
            let _ = self.set_pixel(cx, cy, c);
            return;
        }

        let a2 = i64::from(a) * i64::from(a);
        let b2 = i64::from(b) * i64::from(b);

        let mut x: i64 = 0;
        let mut y: i64 = i64::from(b);
        let mut d = b2 - a2 * i64::from(b) + a2 / 4;
        while b2 * x <= a2 * y {
            let xx = x as i32;
            let yy = y as i32;
            let _ = self.set_pixel(cx + xx, cy + yy, c);
            let _ = self.set_pixel(cx - xx, cy + yy, c);
            let _ = self.set_pixel(cx + xx, cy - yy, c);
            let _ = self.set_pixel(cx - xx, cy - yy, c);
            if d < 0 {
                d += b2 * (2 * x + 3);
            } else {
                d += b2 * (2 * x + 3) + a2 * (-2 * y + 2);
                y -= 1;
            }
            x += 1;
        }

        x = i64::from(a);
        y = 0;
        d = a2 - b2 * i64::from(a) + b2 / 4;
        while a2 * y <= b2 * x {
            let xx = x as i32;
            let yy = y as i32;
            let _ = self.set_pixel(cx + xx, cy + yy, c);
            let _ = self.set_pixel(cx - xx, cy + yy, c);
            let _ = self.set_pixel(cx + xx, cy - yy, c);
            let _ = self.set_pixel(cx - xx, cy - yy, c);
            if d < 0 {
                d += a2 * (2 * y + 3);
            } else {
                d += a2 * (2 * y + 3) + b2 * (-2 * x + 2);
                x -= 1;
            }
            y += 1;
        }
    }

    // Filled ellipse using horizontal spans
    pub fn elli(&mut self, cx: i32, cy: i32, a: i32, b: i32, color: u8) {
        if a < 0 || b < 0 {
            return;
        }
        let c = color & COLOR_MASK;
        if a == 0 && b == 0 {
            let _ = self.set_pixel(cx, cy, c);
            return;
        }
        if a == 0 {
            for yy in (cy - b)..=(cy + b) {
                let _ = self.set_pixel(cx, yy, c);
            }
            return;
        }
        if b == 0 {
            for xx in (cx - a)..=(cx + a) {
                let _ = self.set_pixel(xx, cy, c);
            }
            return;
        }
        let af = a as f32;
        let bf = b as f32;
        let bf2 = bf * bf;
        for dy in -b..=b {
            let yf = dy as f32;
            let t = 1.0 - (yf * yf) / bf2;
            if t < 0.0 {
                continue;
            }
            let xf = af * t.sqrt();
            let x = xf.floor() as i32;
            self.hspan(cx - x, cx + x, cy + dy, c);
        }
    }

    // Triangle border via lines
    #[allow(clippy::too_many_arguments)]
    pub fn trib(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: u8) {
        let c = color & 0x0F;
        self.line(x1, y1, x2, y2, c);
        self.line(x2, y2, x3, y3, c);
        self.line(x3, y3, x1, y1, c);
    }

    // Filled triangle using scanline rasterization
    #[allow(clippy::too_many_arguments)]
    pub fn tri(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: u8) {
        let c = color & 0x0F;
        // Convert to CCW orientation for consistent edge tests
        let area =
            i64::from(x1 - x0) * i64::from(y2 - y0) - i64::from(x2 - x0) * i64::from(y1 - y0);
        let (v0x, v0y, v1x, v1y, v2x, v2y) = if area < 0 {
            (x0, y0, x2, y2, x1, y1)
        } else {
            (x0, y0, x1, y1, x2, y2)
        };

        // Bounding box (exclusive max per top-left rule)
        let min_x = v0x.min(v1x).min(v2x);
        let min_y = v0y.min(v1y).min(v2y);
        let max_x = v0x.max(v1x).max(v2x);
        let max_y = v0y.max(v1y).max(v2y);

        // Doubling coordinates to evaluate edge functions at pixel centers (x+0.5, y+0.5)
        let (ax2, ay2) = (i64::from(v0x) * 2, i64::from(v0y) * 2);
        let (bx2, by2) = (i64::from(v1x) * 2, i64::from(v1y) * 2);
        let (cx2, cy2) = (i64::from(v2x) * 2, i64::from(v2y) * 2);

        // Edge deltas
        let e0_dx = bx2 - ax2;
        let e0_dy = by2 - ay2; // v0->v1
        let e1_dx = cx2 - bx2;
        let e1_dy = cy2 - by2; // v1->v2
        let e2_dx = ax2 - cx2;
        let e2_dy = ay2 - cy2; // v2->v0

        // Top-left classification
        let e0_top_left = e0_dy > 0 || (e0_dy == 0 && e0_dx < 0);
        let e1_top_left = e1_dy > 0 || (e1_dy == 0 && e1_dx < 0);
        let e2_top_left = e2_dy > 0 || (e2_dy == 0 && e2_dx < 0);

        // Iterate over pixels in bounding box with top-left rule: y in [min_y, max_y), x in [min_x, max_x)
        for y in min_y..max_y {
            for x in min_x..max_x {
                let px = i64::from(x) * 2 + 1;
                let py = i64::from(y) * 2 + 1;
                // Edge functions
                let e0 = (py - ay2) * e0_dx - (px - ax2) * e0_dy;
                let e1 = (py - by2) * e1_dx - (px - bx2) * e1_dy;
                let e2 = (py - cy2) * e2_dx - (px - cx2) * e2_dy;

                // Apply top-left inclusion rules
                if (e0 > 0 || (e0 == 0 && e0_top_left))
                    && (e1 > 0 || (e1 == 0 && e1_top_left))
                    && (e2 > 0 || (e2 == 0 && e2_top_left))
                {
                    let _ = self.set_pixel(x, y, c);
                }
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
        let cidx = color & COLOR_MASK;
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

            let (start_col, width_cols) = if fixed {
                (0, GLYPH_W)
            } else {
                // Variable-width: trim empty columns using LSB-left orientation
                let mut left = GLYPH_W;
                let mut right = 0;
                for row in 0..GLYPH_H {
                    let mask = font[base + row];
                    if mask != 0 {
                        // find first 1 from the left (LSB)
                        let mut l = 0;
                        while l < GLYPH_W && ((mask >> l) & 1) == 0 {
                            l += 1;
                        }
                        // find last 1 from the left (rightmost set bit + 1)
                        let mut r = GLYPH_W;
                        while r > 0 && (((mask >> (r - 1)) & 1) == 0) {
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
            };

            // Draw glyph
            for row in 0..GLYPH_H {
                let mask = font[base + row];
                for col in 0..width_cols {
                    let bit_idx = start_col + col;
                    if bit_idx < GLYPH_W && (((mask >> bit_idx) & 1) != 0) {
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
            if fixed {
                pos += ADV * scale;
            } else if width_cols > 0 {
                pos += ((width_cols as i32) + 1) * scale;
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

#[must_use]
pub const fn dimensions() -> (u32, u32) {
    (Framebuffer::WIDTH, Framebuffer::HEIGHT)
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}
