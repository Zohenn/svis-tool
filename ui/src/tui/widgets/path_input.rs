use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::{style::*, text::Line};

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
    fn bound_state(&self) -> Option<FocusableWidget> {
        Some(FocusableWidget::PathInput)
    }

    fn render<'widget, 'app: 'widget>(&self, mut context: RenderContext<'app, '_>, rect: Rect) {
        let label = Line::from(keybindings!("p""ath"));

        let input = InputWidget::new(context.is_focused()).label(label);

        let (app, frame) = context.app_frame_mut();

        InputWidget::frame_render(frame, input, rect, &mut app.path_state.path_input);
    }
}
