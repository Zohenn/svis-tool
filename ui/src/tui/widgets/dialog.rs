use ratatui::{
    layout::Rect,
    prelude::*,
    style::{Color, Style},
};

#[derive(Default)]
pub struct Dialog<State> {
    pub state: State,
    pub open: bool,
}

impl<State: DialogContent> Dialog<State> {
    pub fn render_dialog(&mut self, f: &mut Frame, area: Rect) {
        if !self.open {
            return;
        }

        let buffer = f.buffer_mut();

        buffer.set_style(*buffer.area(), Style::default().fg(Color::DarkGray));
        self.state.render_content(f, area);
    }
}

pub trait DialogContent {
    fn render_content(&mut self, f: &mut Frame, area: Rect);
}
