use ratatui::{
    layout::Rect,
    prelude::*,
    style::{Color, Style},
};

pub trait DialogContent {
    fn render_content(&mut self, f: &mut Frame, area: Rect);

    fn render_dialog(&mut self, f: &mut Frame, area: Rect, open: bool) {
        if !open {
            return;
        }

        let buffer = f.buffer_mut();

        buffer.set_style(*buffer.area(), Style::default().fg(Color::DarkGray));
        self.render_content(f, area);
    }
}
