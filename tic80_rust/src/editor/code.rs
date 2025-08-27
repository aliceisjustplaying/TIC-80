use ropey::Rope;

#[derive(Copy, Clone, Debug)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct CodeBuffer {
    rope: Rope,
    pub caret_line: usize,
    pub caret_col: usize,
    pub scroll_line: usize,
    pub scroll_col: usize,
}

impl CodeBuffer {
    #[must_use]
    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            caret_line: 0,
            caret_col: 0,
            scroll_line: 0,
            scroll_col: 0,
        }
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    #[must_use]
    pub fn line_len(&self, line: usize) -> usize {
        if line >= self.rope.len_lines() {
            0
        } else {
            self.rope.line(line).len_chars()
        }
    }

    pub fn move_left(&mut self) {
        if self.caret_col > 0 {
            self.caret_col -= 1;
        } else if self.caret_line > 0 {
            self.caret_line -= 1;
            self.caret_col = self.line_len(self.caret_line);
        }
    }

    pub fn move_right(&mut self) {
        let len = self.line_len(self.caret_line);
        if self.caret_col < len {
            self.caret_col += 1;
        } else if self.caret_line + 1 < self.line_count() {
            self.caret_line += 1;
            self.caret_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.caret_line > 0 {
            self.caret_line -= 1;
            let len = self.line_len(self.caret_line);
            if self.caret_col > len {
                self.caret_col = len;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.caret_line + 1 < self.line_count() {
            self.caret_line += 1;
            let len = self.line_len(self.caret_line);
            if self.caret_col > len {
                self.caret_col = len;
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn ensure_visible(&mut self, lines_visible: usize, cols_visible: usize) {
        if self.caret_line < self.scroll_line {
            self.scroll_line = self.caret_line;
        } else if self.caret_line >= self.scroll_line + lines_visible {
            self.scroll_line = self
                .caret_line
                .saturating_sub(lines_visible.saturating_sub(1));
        }
        if self.caret_col < self.scroll_col {
            self.scroll_col = self.caret_col;
        } else if self.caret_col >= self.scroll_col + cols_visible {
            self.scroll_col = self
                .caret_col
                .saturating_sub(cols_visible.saturating_sub(1));
        }
    }

    pub fn draw(&mut self, fb: &mut crate::gfx::framebuffer::Framebuffer, area: Area) {
        let gutter_w = 24i32;
        let lines_vis = (area.h / 8).max(1) as usize;
        let cols_vis = ((area.w - gutter_w) / 6).max(1) as usize;
        self.ensure_visible(lines_vis, cols_vis);

        // Clip to drawing area
        fb.clip(area.x, area.y, area.w, area.h);

        // Background is already drawn by UI; render gutter and text
        for i in 0..lines_vis {
            let line_idx = self.scroll_line + i;
            if line_idx >= self.line_count() {
                break;
            }
            // Gutter (1-based line numbers)
            let gutter_y = area.y + i32::try_from(i).unwrap_or(0) * 8;
            let ln = line_idx + 1;
            let label = format!("{ln:>3}");
            let _ = fb.print_text(&label, area.x + 2, gutter_y, 6, true, 1, false);

            // Text slice
            let mut line = self.rope.line(line_idx).to_string();
            if line.ends_with('\n') {
                line.pop();
            }
            let start = self.scroll_col.min(line.chars().count());
            let mut iter = line.chars().skip(start);
            let vis: String = iter.by_ref().take(cols_vis).collect();
            // Monospace rendering for alignment (fixed=true)
            let _ = fb.print_text(&vis, area.x + gutter_w, gutter_y, 12, true, 1, false);
        }

        // Caret (TIC-80 style: red box slightly larger than glyph, with 1px drop shadow; underlying glyph drawn dark)
        if self.caret_line >= self.scroll_line && self.caret_line < self.scroll_line + lines_vis {
            let row = i32::try_from(self.caret_line - self.scroll_line).unwrap_or(0);
            let col = i32::try_from(self.caret_col.saturating_sub(self.scroll_col)).unwrap_or(0);
            let cell_x = area.x + gutter_w + col * 6;
            let cell_y = area.y + row * 8;
            // Target a box that is ~1px larger than the glyph horizontally and vertically
            // within the 6x8 cell: width 7 (left-extended by 1), height 7, leaving room for a 1px bottom shadow.
            let mut fill_x = cell_x - 1; // extend 1px to the left
            let fill_y = (cell_y - 1).max(area.y); // shift up by 1px
            let mut fill_w = 7; // cover 6px cell width + 1px extra on left
            let fill_h = 7; // leave 1px bottom for shadow
                            // Clamp fill to the code pane (don’t bleed into gutter if at first column)
            let code_left = area.x + gutter_w;
            if fill_x < code_left {
                let delta = code_left - fill_x;
                fill_x = code_left;
                fill_w = (fill_w - delta).max(1);
            }
            // Fill: palette 8 (red)
            fb.rect(fill_x, fill_y, fill_w, fill_h, 8);
            // Shadow (palette 0) to the right and along bottom
            fb.rect(fill_x + fill_w, fill_y, 1, fill_h, 0);
            fb.rect(fill_x + 1, fill_y + fill_h, fill_w, 1, 0);

            // Draw the underlying glyph in dark color to simulate inversion
            let line_idx = self.caret_line;
            if line_idx < self.line_count() {
                let mut full = self.rope.line(line_idx).to_string();
                if full.ends_with('\n') {
                    full.pop();
                }
                let total = full.chars().count();
                let start = self.scroll_col.min(total);
                let idx = start + (col as usize);
                if idx < total {
                    let ch = full.chars().nth(idx).unwrap_or(' ');
                    let s = ch.to_string();
                    // Render in color 0 (dark) monospaced aligned to cell; this simulates inversion
                    let _ = fb.print_text(&s, cell_x, cell_y, 0, true, 1, false);
                }
            }
        }

        fb.clip_reset();
    }
}
