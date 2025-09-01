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
    // Selection anchor as (line, col) if active
    sel_anchor: Option<(usize, usize)>,
    // Undo/redo stacks (each EditOp can be a batch of atomic edits)
    undo: Vec<EditOp>,
    redo: Vec<EditOp>,
}

#[derive(Clone, Debug)]
enum EditKind {
    Insert { index: usize, text: String },
    Delete { index: usize, text: String },
}

#[derive(Clone, Debug, Default)]
struct EditOp {
    ops: Vec<EditKind>,
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
            sel_anchor: None,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    #[must_use]
    fn caret_char_index(&self) -> usize {
        let base = self.rope.line_to_char(self.caret_line);
        let col = self.caret_col.min(self.line_len(self.caret_line));
        base + col
    }

    pub fn insert_char(&mut self, ch: char) {
        if ch == '\r' {
            return;
        }
        if ch == '\n' {
            self.insert_newline();
            return;
        }
        if self.has_selection() {
            let text = ch.to_string();
            self.replace_selection_with(&text);
            return;
        }
        let idx = self.caret_char_index();
        self.rope.insert_char(idx, ch);
        self.push_undo(EditKind::Insert {
            index: idx,
            text: ch.to_string(),
        });
        self.clear_redo();
        self.set_caret_at_index(idx + 1);
    }

    pub fn insert_tab(&mut self) {
        // Default to a single space for compact 240x136 layout
        if self.has_selection() {
            self.replace_selection_with(" ");
            return;
        }
        let idx = self.caret_char_index();
        self.rope.insert(idx, " ");
        self.push_undo(EditKind::Insert {
            index: idx,
            text: " ".to_string(),
        });
        self.clear_redo();
        self.set_caret_at_index(idx + 1);
    }

    pub fn insert_newline(&mut self) {
        if self.has_selection() {
            self.replace_selection_with("\n");
            return;
        }
        let idx = self.caret_char_index();
        self.rope.insert_char(idx, '\n');
        self.push_undo(EditKind::Insert {
            index: idx,
            text: "\n".to_string(),
        });
        self.clear_redo();
        self.set_caret_at_index(idx + 1);
    }

    pub fn backspace(&mut self) {
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range_idx() {
                let deleted = self.delete_range(start, end);
                self.push_undo(EditKind::Delete {
                    index: start,
                    text: deleted,
                });
                self.clear_redo();
            }
            return;
        }
        if self.caret_col > 0 {
            let idx = self.caret_char_index();
            if idx > 0 {
                let removed = self.slice_to_string(idx - 1, idx);
                self.rope.remove(idx - 1..idx);
                self.push_undo(EditKind::Delete {
                    index: idx - 1,
                    text: removed,
                });
                self.clear_redo();
                self.set_caret_at_index(idx - 1);
            }
        } else if self.caret_line > 0 {
            // Merge with previous line (remove the preceding newline)
            let idx = self.caret_char_index();
            if idx > 0 {
                let removed = self.slice_to_string(idx - 1, idx);
                self.rope.remove(idx - 1..idx);
                self.push_undo(EditKind::Delete {
                    index: idx - 1,
                    text: removed,
                });
                self.clear_redo();
                // caret moves to start index (previous line end)
                self.set_caret_at_index(idx - 1);
            }
        }
    }

    pub fn delete_forward(&mut self) {
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range_idx() {
                let deleted = self.delete_range(start, end);
                self.push_undo(EditKind::Delete {
                    index: start,
                    text: deleted,
                });
                self.clear_redo();
            }
            return;
        }
        let idx = self.caret_char_index();
        if idx < self.rope.len_chars() {
            // Deleting forward: remove current char or join next line when at EOL
            let removed = self.slice_to_string(idx, idx + 1);
            self.rope.remove(idx..=idx);
            self.push_undo(EditKind::Delete {
                index: idx,
                text: removed,
            });
            self.clear_redo();
            // Caret stays; adjust if we deleted a newline (join lines)
            self.set_caret_at_index(idx);
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn home(&mut self) {
        self.caret_col = 0;
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn end(&mut self) {
        self.caret_col = self.line_len(self.caret_line);
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

    #[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
    pub fn draw(&mut self, fb: &mut crate::gfx::framebuffer::Framebuffer, area: Area) {
        // Gutter width: 3 digits * 6px = 18px, plus a 1px gap before code
        let gutter_w = 18i32;
        let gap = 1i32;
        // Match TIC-80 editor line pitch: 7px (TIC_FONT_HEIGHT + 1)
        let line_pitch = 7i32;
        let lines_vis = (area.h / line_pitch).max(1) as usize;
        let cols_vis = ((area.w - gutter_w - gap) / 6).max(1) as usize;
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
            let gutter_y = area.y + i32::try_from(i).unwrap_or(0) * line_pitch;
            let ln = line_idx + 1;
            let label = format!("{ln:>3}");
            // Right-justified 3-digit label, drawn flush to gutter (no extra left/right padding)
            let _ = fb.print_text(&label, area.x, gutter_y, 14, true, 1, true);

            // Text slice (track newline separately to handle selection over EOL/empty lines)
            let mut src_line = self.rope.line(line_idx).to_string();
            let had_nl = src_line.ends_with('\n');
            if had_nl {
                src_line.pop();
            }
            let line = src_line;
            let start = self.scroll_col.min(line.chars().count());
            let mut iter = line.chars().skip(start);
            let vis: String = iter.by_ref().take(cols_vis).collect();
            // Draw characters cell-by-cell, applying selection overlays where needed
            let line_char_start = self.rope.line_to_char(line_idx);
            let sel = self.selection_range_idx();
            for (i_vis, ch) in vis.chars().enumerate() {
                let cell_x = area.x + gutter_w + gap + i32::try_from(i_vis).unwrap_or(0) * 6;
                let cell_y = gutter_y;
                let global_idx = line_char_start + start + i_vis;
                let selected = sel.is_some_and(|(s, e)| global_idx >= s && global_idx < e);
                if selected {
                    // Shadow and fill per TIC-80
                    fb.rect(cell_x, cell_y, 7, 7, 0);
                    // selection fill uses theme SELECT color (default 14)
                    fb.rect(cell_x - 1, cell_y - 1, 7, 7, 14);
                    // Dark glyph on top
                    let s = ch.to_string();
                    let _ = fb.print_text(&s, cell_x, cell_y, 15, true, 1, true);
                } else {
                    // Normal glyph (no selection overlay)
                    let s = ch.to_string();
                    // TIC default text color (no syntax) is white (12), 6px tall
                    let _ = fb.print_text(&s, cell_x, cell_y, 12, true, 1, true);
                }
            }
            // Extra selection cell for newline (EOL) when selected
            if had_nl {
                if let Some((sel_start, sel_end)) = sel {
                    let display_len = line.chars().count();
                    let nl_idx = line_char_start + display_len; // index of '\n' in rope for this line
                    if sel_start <= nl_idx && sel_end > nl_idx {
                        let nl_col = i32::try_from(display_len).unwrap_or(0);
                        let col_vis = nl_col - self.scroll_col as i32;
                        if col_vis >= 0 && col_vis < cols_vis as i32 {
                            let cell_x = area.x + gutter_w + gap + col_vis * 6;
                            let cell_y = gutter_y;
                            fb.rect(cell_x, cell_y, 7, 7, 0);
                            fb.rect(cell_x - 1, cell_y - 1, 7, 7, 14);
                        }
                    }
                }
            }
        }

        // Caret (red box aligned to 6x8 cell, with 1px drop shadow; underlying glyph drawn dark)
        if self.caret_line >= self.scroll_line && self.caret_line < self.scroll_line + lines_vis {
            let row = i32::try_from(self.caret_line - self.scroll_line).unwrap_or(0);
            let col = i32::try_from(self.caret_col.saturating_sub(self.scroll_col)).unwrap_or(0);
            let cell_x = area.x + gutter_w + gap + col * 6;
            let cell_y = area.y + row * line_pitch;
            // TIC-80 caret style: drop shadow rect (black) then caret rect (cursor color, default 2), both 7x7, offset by 1px
            fb.rect(cell_x, cell_y, 7, 7, 0);
            fb.rect(cell_x - 1, cell_y - 1, 7, 7, 2);

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
                    // Render underlying glyph in background color to simulate inversion
                    let _ = fb.print_text(&s, cell_x, cell_y, 15, true, 1, true);
                }
            }
        }

        fb.clip_reset();
    }

    #[must_use]
    pub fn as_string(&self) -> String {
        self.rope.to_string()
    }

    // Selection helpers
    #[allow(clippy::missing_const_for_fn)]
    pub fn clear_selection(&mut self) {
        self.sel_anchor = None;
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn start_selection(&mut self) {
        self.sel_anchor = Some((self.caret_line, self.caret_col));
    }

    pub fn ensure_selection_anchor(&mut self) {
        if self.sel_anchor.is_none() {
            self.start_selection();
        }
    }

    #[must_use]
    pub fn has_selection(&self) -> bool {
        self.selection_range_idx().is_some()
    }

    #[must_use]
    pub fn selection_range_idx(&self) -> Option<(usize, usize)> {
        let (al, ac) = self.sel_anchor?;
        let a = self.line_col_to_index(al, ac);
        let b = self.caret_char_index();
        match a.cmp(&b) {
            core::cmp::Ordering::Equal => None,
            core::cmp::Ordering::Less => Some((a, b)),
            core::cmp::Ordering::Greater => Some((b, a)),
        }
    }

    pub fn select_all(&mut self) {
        self.sel_anchor = Some((0, 0));
        let end_idx = self.rope.len_chars();
        self.set_caret_at_index(end_idx);
    }

    // Clipboard-friendly operations (pure text; caller integrates with OS clipboard)
    #[must_use]
    pub fn copy_selection_text(&self) -> Option<String> {
        let (s, e) = self.selection_range_idx()?;
        Some(self.slice_to_string(s, e))
    }

    pub fn cut_selection_text(&mut self) -> Option<String> {
        let (s, e) = self.selection_range_idx()?;
        let deleted = self.delete_range(s, e);
        self.push_undo(EditKind::Delete {
            index: s,
            text: deleted.clone(),
        });
        self.clear_redo();
        Some(deleted)
    }

    pub fn paste_text(&mut self, text: &str) {
        if self.has_selection() {
            if let Some((s, e)) = self.selection_range_idx() {
                let mut op = EditOp { ops: Vec::new() };
                let deleted = self.delete_range(s, e);
                op.ops.push(EditKind::Delete {
                    index: s,
                    text: deleted,
                });
                self.insert_text_at(s, text);
                op.ops.push(EditKind::Insert {
                    index: s,
                    text: text.to_string(),
                });
                self.undo.push(op);
            }
        } else {
            let idx = self.caret_char_index();
            self.insert_text_at(idx, text);
            self.push_undo(EditKind::Insert {
                index: idx,
                text: text.to_string(),
            });
        }
        self.clear_redo();
    }

    pub fn undo(&mut self) {
        if let Some(op) = self.undo.pop() {
            let redo_op = op.clone();
            // Apply inverse in reverse order
            for k in op.ops.iter().rev() {
                match k {
                    EditKind::Insert { index, text } => {
                        let s = *index;
                        let e = s + text.chars().count();
                        let _ = self.delete_range(s, e);
                        self.set_caret_at_index(*index);
                    }
                    EditKind::Delete { index, text } => {
                        self.insert_text_at(*index, text);
                        self.set_caret_at_index(index + text.chars().count());
                    }
                }
            }
            self.redo.push(redo_op);
            self.clear_selection();
        }
    }

    pub fn redo(&mut self) {
        if let Some(op) = self.redo.pop() {
            let undo_op = op.clone();
            for k in &op.ops {
                match k {
                    EditKind::Insert { index, text } => {
                        self.insert_text_at(*index, text);
                        self.set_caret_at_index(index + text.chars().count());
                    }
                    EditKind::Delete { index, text } => {
                        let s = *index;
                        let e = s + text.chars().count();
                        let _ = self.delete_range(s, e);
                        self.set_caret_at_index(*index);
                    }
                }
            }
            self.undo.push(undo_op);
            self.clear_selection();
        }
    }

    // Internals --------------------------------------------------------------------------------
    fn push_undo(&mut self, k: EditKind) {
        self.undo.push(EditOp { ops: vec![k] });
    }
    fn clear_redo(&mut self) {
        self.redo.clear();
    }
    fn line_col_to_index(&self, line: usize, col: usize) -> usize {
        let base = self.rope.line_to_char(line);
        base + col.min(self.line_len(line))
    }
    fn set_caret_at_index(&mut self, idx: usize) {
        let line = self.rope.char_to_line(idx.min(self.rope.len_chars()));
        let base = self.rope.line_to_char(line);
        self.caret_line = line;
        self.caret_col = idx.saturating_sub(base);
    }
    fn slice_to_string(&self, s: usize, e: usize) -> String {
        self.rope.slice(s..e).to_string()
    }
    fn delete_range(&mut self, s: usize, e: usize) -> String {
        let text = self.slice_to_string(s, e);
        self.rope.remove(s..e);
        self.set_caret_at_index(s);
        self.clear_selection();
        text
    }
    fn insert_text_at(&mut self, idx: usize, text: &str) {
        self.rope.insert(idx, text);
        let new_idx = idx + text.chars().count();
        self.set_caret_at_index(new_idx);
        self.clear_selection();
    }
    fn replace_selection_with(&mut self, text: &str) {
        if let Some((s, e)) = self.selection_range_idx() {
            let mut op = EditOp { ops: Vec::new() };
            let deleted = self.delete_range(s, e);
            op.ops.push(EditKind::Delete {
                index: s,
                text: deleted,
            });
            self.insert_text_at(s, text);
            op.ops.push(EditKind::Insert {
                index: s,
                text: text.to_string(),
            });
            self.undo.push(op);
            self.clear_redo();
        } else {
            self.paste_text(text);
        }
    }
}
