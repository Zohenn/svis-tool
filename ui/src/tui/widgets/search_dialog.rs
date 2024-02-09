use crate::tui::{
    core::{FocusableWidgetState, HandleEventResult},
    widget_utils::CustomStyles,
    FocusableWidget,
};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, Padding},
};

use crate::{keybindings, tui::widget_utils::default_block};

use super::{
    dialog::{Dialog, DialogContent},
    input::{InputWidget, InputWidgetState},
};

#[derive(Default)]
pub struct SearchDialogState {
    pub path_input: InputWidgetState,
}

impl DialogContent for SearchDialogState {
    fn render_content(&mut self, f: &mut Frame, area: Rect) {
        let vertical_chunk = Layout::vertical([Constraint::Ratio(4, 5)])
            .flex(Flex::Center)
            .split(area)[0];

        let chunk = Layout::horizontal([Constraint::Min(3)]).split(vertical_chunk)[0];

        let buffer = f.buffer_mut();

        for x in chunk.left()..chunk.right() {
            for y in chunk.top()..chunk.bottom() {
                buffer.get_mut(x, y).reset();
            }
        }

        let block = default_block().bg(Color::Black).padding(Padding::uniform(1));

        f.render_widget(block, chunk);

        let label = Line::from(keybindings!("p""ath"));

        let input = InputWidget::new(true).label(label);

        InputWidget::frame_render(f, input, chunk, &mut self.path_input);
    }
}

impl FocusableWidgetState for Dialog<SearchDialogState> {
    fn handle_events(&mut self, event: crossterm::event::KeyEvent) -> HandleEventResult {
        self.open = false;
        HandleEventResult::ChangeFocus(FocusableWidget::FileList)
    }
}
