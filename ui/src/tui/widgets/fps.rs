use std::time::{Duration, Instant};

use ratatui::{buffer::Buffer, layout::Rect, text::Text, widgets::Widget};

// Copied from ratatui example
pub struct FpsWidget {
    visible: bool,
    frame_count: usize,
    last_instant: Instant,
    fps: Option<f32>,
}

impl Default for FpsWidget {
    fn default() -> Self {
        Self {
            visible: false,
            frame_count: 0,
            last_instant: Instant::now(),
            fps: None,
        }
    }
}

impl Widget for &mut FpsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.calculate_fps();

        if !self.visible {
            return;
        }

        if let Some(fps) = self.fps {
            let text = format!("{:.1} fps", fps);
            Text::from(text).render(area, buf);
        }
    }
}

impl FpsWidget {
    fn calculate_fps(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_instant.elapsed();
        if elapsed > Duration::from_secs(1) && self.frame_count > 2 {
            self.fps = Some(self.frame_count as f32 / elapsed.as_secs_f32());
            self.frame_count = 0;
            self.last_instant = Instant::now();
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}
