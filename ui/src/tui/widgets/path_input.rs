use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::Rect, style::*, text::Line, Frame};

use crate::keybindings;

use crate::tui::{
    core::{FocusableWidgetState, HandleEventResult},
    widget_utils::CustomStyles,
    App, FocusableWidget,
};

use super::input::{InputWidget, InputWidgetState};

#[derive(Default)]
pub struct PathState {
    pub path_input: InputWidgetState,
}

impl FocusableWidgetState for PathState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match event.code {
            KeyCode::Enter => HandleEventResult::Callback(Self::callback),
            _ => {
                return self.path_input.handle_events(event);
            }
        }
    }

    fn callback(app: &mut App) -> HandleEventResult {
        let path = app.path_state.path_input.value().to_owned();

        app.file_list_state.analyze_path(path);

        HandleEventResult::ChangeFocus(FocusableWidget::FileList)
    }
}

pub fn render_path_input(f: &mut Frame, app: &mut App, rect: Rect) {
    let is_focused = matches!(app.focused_widget, Some(FocusableWidget::PathInput));

    let label = Line::from(keybindings!("p""ath"));

    let input = InputWidget::new(is_focused).label(label);

    InputWidget::frame_render(f, input, rect, &mut app.path_state.path_input);
}
