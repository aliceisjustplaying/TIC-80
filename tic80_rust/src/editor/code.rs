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

    #[allow(clippy::cast_possible_truncation)]
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
            // Selection highlight for this line (with TIC-80-style drop shadow)
            if let Some((sel_start, sel_end)) = self.selection_range_idx() {
                // Compute selection coverage in columns for this visible segment
                let line_char_start = self.rope.line_to_char(line_idx);
                let line_char_end = line_char_start + self.line_len(line_idx);
                let s = sel_start.max(line_char_start);
                let e = sel_end.min(line_char_end);
                if e > s {
                    let a = (s - line_char_start) as i32;
                    let b = (e - line_char_start) as i32;
                    let a_vis = (a - self.scroll_col as i32).max(0);
                    let b_vis = (b - self.scroll_col as i32).max(0);
                    let from = a_vis.min(cols_vis as i32).max(0);
                    let to = b_vis.min(cols_vis as i32).max(from);
                    if to > from {
                        let sel_x = area.x + gutter_w + from * 6;
                        let y_top = (gutter_y - 1).max(area.y);
                        let sel_w = (to - from) * 6;
                        // Fill (7px tall), like caret box
                        fb.rect(sel_x, y_top, sel_w, 7, 14);
                        // Decide whether to draw the right shadow for this row segment.
                        // Only draw if the next line's selection doesn't extend as far right (outer perimeter).
                        let mut draw_right_shadow = true;
                        if line_idx + 1 < self.line_count() {
                            let next_line_char_start = self.rope.line_to_char(line_idx + 1);
                            let next_line_char_end = next_line_char_start + self.line_len(line_idx + 1);
                            // Next line selection coverage
                            let ns = sel_start.max(next_line_char_start);
                            let ne = sel_end.min(next_line_char_end);
                            if ne > ns {
                                let na = (ns.saturating_sub(next_line_char_start)) as i32;
                                let nb = (ne.saturating_sub(next_line_char_start)) as i32;
                                let na_vis = (na - self.scroll_col as i32).max(0);
                                let nb_vis = (nb - self.scroll_col as i32).max(0);
                                let nfrom = na_vis.min(cols_vis as i32).max(0);
                                let nto = nb_vis.min(cols_vis as i32).max(nfrom);
                                // If next line's right edge is strictly greater than this line's,
                                // skip right shadow here (it's interior to the overall blob).
                                // Equal width should draw to produce a continuous vertical edge.
                                if nto > to {
                                    draw_right_shadow = false;
                                }
                            }
                        }
                        if draw_right_shadow {
                            // Right edge: start at the same top as fill and span 8px so adjacent rows abut exactly
                            fb.rect(sel_x + sel_w, y_top, 1, 8, 0);
                        }
                        // Only draw bottom shadow if selection does not continue to next line
                        let continues_down = sel_end >= line_char_end;
                        if !continues_down {
                            fb.rect(sel_x, y_top + 7, sel_w, 1, 0);
                        }
                    }
                }
            }
            // Monospace rendering for alignment (fixed=true)
            let _ = fb.print_text(&vis, area.x + gutter_w, gutter_y, 12, true, 1, false);
        }

        // Caret (red box aligned to 6x8 cell, with 1px drop shadow; underlying glyph drawn dark)
        if self.caret_line >= self.scroll_line && self.caret_line < self.scroll_line + lines_vis {
            let row = i32::try_from(self.caret_line - self.scroll_line).unwrap_or(0);
            let col = i32::try_from(self.caret_col.saturating_sub(self.scroll_col)).unwrap_or(0);
            let cell_x = area.x + gutter_w + col * 6;
            let cell_y = area.y + row * 8;
            // Box aligned to cell: width 6, height 7 (reserve 1px bottom for shadow)
            let fill_x = cell_x;
            let fill_y = (cell_y - 1).max(area.y);
            let fill_w = 6;
            let fill_h = 7;
            // Fill: palette 8 (red)
            fb.rect(fill_x, fill_y, fill_w, fill_h, 8);
            // Shadow (palette 0) to the right and along bottom
            fb.rect(fill_x + fill_w, fill_y, 1, fill_h, 0);
            fb.rect(fill_x, fill_y + fill_h, fill_w, 1, 0);

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
