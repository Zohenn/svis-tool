use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::Rect, style::*, text::Line, Frame};

use crate::keybindings;

use crate::tui::core::custom_widget::{CustomWidget, RenderContext};
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
            KeyCode::Enter => HandleEventResult::Callback(Box::new(Self::callback)),
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

pub struct PathInputWidget;

impl CustomWidget for PathInputWidget {
    type Data = ();

    fn render<'widget, 'app: 'widget>(self, mut context: RenderContext<'app, '_, Self::Data>, rect: Rect) {
        let label = Line::from(keybindings!("p""ath"));

        let input = InputWidget::new(context.is_focused()).label(label);

        let (app, frame) = context.app_frame_mut();

        InputWidget::frame_render(frame, input, rect, &mut app.path_state.path_input);
    }
}

pub fn render_path_input(f: &mut Frame, app: &mut App, rect: Rect) {
    let is_focused = matches!(app.focused_widget, Some(FocusableWidget::PathInput));

    let label = Line::from(keybindings!("p""ath"));

    let input = InputWidget::new(is_focused).label(label);

    InputWidget::frame_render(f, input, rect, &mut app.path_state.path_input);
}
