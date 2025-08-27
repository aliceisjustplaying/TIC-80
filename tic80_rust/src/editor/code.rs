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

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

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
}

