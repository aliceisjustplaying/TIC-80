use crate::gfx::framebuffer::Framebuffer;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tab {
    Code,
    Console,
}

#[derive(Copy, Clone, Debug)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Rect {
    #[allow(clippy::missing_const_for_fn)]
    fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
}

pub struct EditorUi {
    pub active: Tab,
    scale: f64,
    // Hit regions in framebuffer space
    tab_code: Rect,
    tab_console: Rect,
    btn_run: Rect,
    btn_stop: Rect,
    btn_reset: Rect,
}

impl EditorUi {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(scale: f64) -> Self {
        // Tabs (static for now)
        let tab_code = Rect {
            x: 4,
            y: 2,
            w: 40,
            h: 10,
        };
        let tab_console = Rect {
            x: 48,
            y: 2,
            w: 60,
            h: 10,
        };

        // Compute button rectangles from label width (ADV=6) plus padding
        let adv: i32 = 6;
        let pad_x: i32 = 4;
        let h: i32 = 10;
        let y: i32 = 2;
        let gap: i32 = 4;
        let right_margin: i32 = 4;

        let run_w = adv * i32::try_from("RUN".len()).unwrap_or(3) + pad_x * 2; // 26
        let stop_w = adv * i32::try_from("STOP".len()).unwrap_or(4) + pad_x * 2; // 32
        let reset_w = adv * i32::try_from("RESET".len()).unwrap_or(5) + pad_x * 2; // 38

        let mut right = 240 - right_margin;
        let btn_reset = Rect {
            x: right - reset_w,
            y,
            w: reset_w,
            h,
        };
        right = btn_reset.x - gap;
        let btn_stop = Rect {
            x: right - stop_w,
            y,
            w: stop_w,
            h,
        };
        right = btn_stop.x - gap;
        let btn_run = Rect {
            x: right - run_w,
            y,
            w: run_w,
            h,
        };

        Self {
            active: Tab::Code,
            scale,
            tab_code,
            tab_console,
            btn_run,
            btn_stop,
            btn_reset,
        }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn scale(&self) -> f64 {
        self.scale
    }

    // Handle a click in framebuffer coordinates
    pub fn on_click_fb(&mut self, x: i32, y: i32) {
        if self.tab_code.contains(x, y) {
            self.active = Tab::Code;
        } else if self.tab_console.contains(x, y) {
            self.active = Tab::Console;
        } else if self.btn_run.contains(x, y) {
            // No-op for now
        } else if self.btn_stop.contains(x, y) || self.btn_reset.contains(x, y) {
            // No-op
        }
    }

    // Map window coordinates (pixels) to framebuffer space, given integer scale
    #[must_use]
    pub fn window_to_fb(&self, wx: f64, wy: f64) -> (i32, i32) {
        let sx = self.scale;
        #[allow(clippy::cast_possible_truncation)]
        let x = (wx / sx).floor() as i32;
        #[allow(clippy::cast_possible_truncation)]
        let y = (wy / sx).floor() as i32;
        (x, y)
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        // Clear
        fb.cls(0);
        // Top bar: 7px tall with 1px margins around 6px text
        let bar_h = 7;
        // TIC-80 draws toolbar in white
        fb.rect(0, 0, 240, bar_h, 12);

        // Tabs (underline only for CODE)
        let (code_col, _cons_col) = match self.active {
            // Use grey underline similar to CODE EDITOR text
            Tab::Code | Tab::Console => (14, 14),
        };
        // Minimal underline only for CODE
        fb.rect(self.tab_code.x, bar_h - 1, self.tab_code.w, 1, code_col);
        // Buttons
        // Buttons: skip heavy boxes in code prototype

        // Labels (using small scale)
        // Left title label: drop shadow 1px (dark grey 15) then grey (14), like "CODE EDITOR"
        let title_x = 4;
        let title_y = 1; // 1px top margin
        let _ = fb.print_text("CODE", title_x + 1, title_y + 1, 15, true, 1, true);
        let _ = fb.print_text("CODE", title_x, title_y, 14, true, 1, true);
        // Center button labels (placeholder, keep white for readability)
        let adv = 6i32;
        let run_tx =
            self.btn_run.x + (self.btn_run.w - adv * i32::try_from("RUN".len()).unwrap_or(3)) / 2;
        let stop_tx = self.btn_stop.x
            + (self.btn_stop.w - adv * i32::try_from("STOP".len()).unwrap_or(4)) / 2;
        let reset_tx = self.btn_reset.x
            + (self.btn_reset.w - adv * i32::try_from("RESET".len()).unwrap_or(5)) / 2;
        let _ = fb.print_text("RUN", run_tx, 1, 14, true, 1, true);
        let _ = fb.print_text("STOP", stop_tx, 1, 14, true, 1, true);
        let _ = fb.print_text("RESET", reset_tx, 1, 14, true, 1, true);

        // Active panel background
        match self.active {
            Tab::Code => {
                // Code area background uses theme BG (default dark grey 15)
                fb.rect(0, bar_h, 240, 136 - bar_h, 15);
            }
            Tab::Console => {
                fb.rect(0, bar_h, 240, 136 - bar_h, 15);
            }
        }
    }
}
