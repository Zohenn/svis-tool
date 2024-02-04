use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{prelude::Rect, style::*, text::Line, widgets::Paragraph, Frame};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::keybindings;

use crate::tui::{
    core::{FocusableWidgetState, HandleEventResult},
    widget_utils::{default_block, CustomStyles},
    App, FocusableWidget,
};

pub struct PathState {
    pub path_input: Input,
}

impl Default for PathState {
    fn default() -> Self {
        Self {
            path_input: Input::default(),
        }
    }
}

impl FocusableWidgetState for PathState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match event.code {
            KeyCode::Enter => HandleEventResult::Callback(Self::callback),
            KeyCode::Esc => HandleEventResult::Blur,
            _ => {
                self.path_input.handle_event(&Event::Key(event));

                HandleEventResult::KeepFocus
            }
        }
    }

    fn callback(app: &mut App) -> HandleEventResult {
        let path = app.path_state.path_input.value().to_owned();

        app.file_list_state.analyze_path(path);

        HandleEventResult::ChangeFocus(FocusableWidget::FileList)
    }
}

pub fn render_path_input(f: &mut Frame, app: &App, rect: Rect) {
    let width = rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor

    let scroll = app.path_state.path_input.visual_scroll(width as usize);
    let is_focused = matches!(app.focused_widget, Some(FocusableWidget::PathInput));

    let label = Line::from(keybindings!("p""ath"));

    let input = Paragraph::new(app.path_state.path_input.value())
        .style(match is_focused {
            true => Style::default().fg(Color::Yellow),
            false => Style::default(),
        })
        .scroll((0, scroll as u16))
        .block(default_block().title(label));
    f.render_widget(input, rect);

    if is_focused {
        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
        f.set_cursor(
            // Put cursor past the end of the input text
            rect.x + ((app.path_state.path_input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            // Move one line down, from the border to the input line
            rect.y + 1,
        )
    }
}
