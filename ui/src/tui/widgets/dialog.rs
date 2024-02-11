use ratatui::{
    layout::{Flex, Rect},
    prelude::*,
    style::{Color, Style},
    widgets::Block,
};

use crate::theme;

pub trait DialogContent {
    fn render_content(&mut self, f: &mut Frame, area: Rect);

    fn vertical_constraints(&self, _area: Rect) -> Constraint {
        Constraint::Percentage(80)
    }

    fn horizontal_constraints(&self, _area: Rect) -> Constraint {
        Constraint::Percentage(80)
    }

    fn modify_block<'block>(&self, block: Block<'block>) -> Block<'block> {
        block
    }

    fn render_dialog(&mut self, f: &mut Frame, area: Rect, open: bool) {
        if !open {
            return;
        }

        let buffer = f.buffer_mut();

        // Imitate backdrop
        buffer.set_style(*buffer.area(), Style::default().fg(Color::DarkGray));

        let vertical_chunk = Layout::vertical([self.vertical_constraints(area)])
            .flex(Flex::Center)
            .split(area)[0];

        let area = Layout::horizontal([self.horizontal_constraints(area)])
            .flex(Flex::Center)
            .split(vertical_chunk)[0];

        // Clear dialog area
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buffer.get_mut(x, y).reset();
            }
        }

        let block = self.modify_block(Block::default().fg(Color::White).bg(theme::BACKGROUND));

        let block_area = block.inner(area);

        f.render_widget(block, area);

        self.render_content(f, block_area);
    }
}
