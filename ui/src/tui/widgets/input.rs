use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    text::Line,
    widgets::{Paragraph, StatefulWidget},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::tui::{
    core::{FocusableWidgetState, HandleEventResult},
    widget_utils::default_block,
};

#[derive(Default)]
pub struct InputWidgetState {
    input: Input,
    scroll: usize,
}

impl InputWidgetState {
    pub fn with_value(mut self, value: String) -> Self {
        self.input = self.input.with_value(value);
        self
    }

    pub fn value(&self) -> &str {
        self.input.value()
    }

    pub fn reset(&mut self) {
        self.input.reset();
    }
}

impl FocusableWidgetState for InputWidgetState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match event.code {
            KeyCode::Esc => HandleEventResult::Blur,
            _ => {
                self.input.handle_event(&Event::Key(event));

                HandleEventResult::KeepFocus
            }
        }
    }
}

pub struct InputWidget<'label> {
    is_focused: bool,
    label: Option<Line<'label>>,
}

impl<'label> InputWidget<'label> {
    pub fn new(is_focused: bool) -> Self {
        InputWidget {
            is_focused,
            label: None,
        }
    }

    pub fn label(mut self, label: Line<'label>) -> Self {
        self.label = Some(label);
        self
    }

    // Static method to render and set cursor, latter requires Frame thus implementing render from
    // Widget/StatefulWidget does not suffice.
    pub fn frame_render(f: &mut Frame, widget: Self, rect: Rect, state: &mut InputWidgetState) {
        let width = rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor

        let scroll = state.input.visual_scroll(width as usize);
        state.scroll = scroll;

        let is_focused = widget.is_focused;

        f.render_stateful_widget(widget, rect, state);

        if is_focused {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                rect.x + ((state.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                // Move one line down, from the border to the input line
                rect.y + 1,
            )
        }
    }
}

impl<'label> StatefulWidget for InputWidget<'label> {
    type State = InputWidgetState;

    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut block = default_block();

        if let Some(label) = self.label {
            block = block.title(label);
        };

        let input = Paragraph::new(state.input.value())
            .style(match self.is_focused {
                true => Style::default().fg(Color::Yellow),
                false => Style::default(),
            })
            .scroll((0, state.scroll as u16))
            .block(block);

        input.render(rect, buf);
    }
}
