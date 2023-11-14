use std::thread;

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    prelude::Rect,
    style::*,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::core::analyze_path;

use super::{
    core::{FocusableWidgetState, HandleEventResult, StatefulList},
    file_list::AnalyzeDoneState,
    AnalyzeState, App, FocusableWidget,
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
        let state_w = app.file_list_state.analyze_state.clone();

        thread::spawn(move || {
            let mut file_infos = Vec::new();
            let mut files_with_errors = Vec::new();
            let mut files_checked = 0;

            *state_w.write().unwrap() = Some(AnalyzeState::Pending(files_checked));

            analyze_path(&path, |file, result| {
                files_checked += 1;
                *state_w.write().unwrap() = Some(AnalyzeState::Pending(files_checked));
                match result {
                    Ok(info) => file_infos.push(info),
                    Err(err) => files_with_errors.push((file.to_owned(), err)),
                }
            })
            .unwrap();

            *state_w.write().unwrap() = Some(AnalyzeState::Done(AnalyzeDoneState {
                files_checked,
                file_infos: StatefulList::with_items(file_infos),
                files_with_errors,
            }));
        });

        HandleEventResult::Blur
    }
}

pub fn render_path_input(f: &mut Frame, app: &App, rect: Rect) {
    let width = rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor

    let scroll = app.path_state.path_input.visual_scroll(width as usize);
    let is_focused = matches!(app.focused_widget, Some(FocusableWidget::PathInput));

    let label = Line::from(vec!["Path (".into(), "p".underlined(), ")".into()]);

    let input = Paragraph::new(app.path_state.path_input.value())
        .style(match is_focused {
            true => Style::default().fg(Color::Yellow),
            false => Style::default(),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title(label));
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