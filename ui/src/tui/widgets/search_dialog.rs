use crate::tui::{
    core::{FocusableWidgetState, HandleEventResult},
    widget_utils::CustomStyles,
    FocusableWidget,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, Padding},
};

use crate::{keybindings, tui::widget_utils::default_block};

use super::{
    dialog::DialogContent,
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

        let area = Layout::horizontal([Constraint::Min(3)]).split(vertical_chunk)[0];

        let buffer = f.buffer_mut();

        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buffer.get_mut(x, y).reset();
            }
        }

        let block = default_block().bg(Color::Black).padding(Padding::uniform(1));

        f.render_widget(block, area);

        let label = Line::from(keybindings!("p""ath"));

        let input = InputWidget::new(true).label(label);

        InputWidget::frame_render(f, input, area, &mut self.path_input);
    }
}

impl FocusableWidgetState for SearchDialogState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match event.code {
            KeyCode::Esc => HandleEventResult::ChangeFocus(FocusableWidget::FileList),
            _ => HandleEventResult::KeepFocus,
        }
    }
}
