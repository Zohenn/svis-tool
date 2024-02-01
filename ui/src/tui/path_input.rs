use std::{sync::atomic::Ordering, thread};

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{prelude::Rect, style::*, text::Line, widgets::Paragraph, Frame};
use tui_input::{backend::crossterm::EventHandler, Input};

use core::analyze_path;

use crate::keybindings;

use super::{
    core::{FocusableWidgetState, HandleEventResult},
    file_list::{AnalyzePendingState, FileInfoType, SourceMappingErrorInfo},
    widget_utils::{default_block, CustomStyles},
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
        let pending_state = AnalyzePendingState::default();
        let files_checked_atomic = pending_state.count.clone();
        let file_infos = pending_state.file_infos.clone();
        let finished_atomic = pending_state.finished.clone();
        app.file_list_state.analyze_state = Some(AnalyzeState::Pending(pending_state));

        thread::spawn(move || {
            let mut local_file_infos = vec![];

            let result = analyze_path(&path, |file, result| {
                files_checked_atomic.fetch_add(1, Ordering::Relaxed);

                match result {
                    Ok(info) => local_file_infos.push(FileInfoType::Info(info)),
                    Err(err) => {
                        local_file_infos.push(FileInfoType::Err(SourceMappingErrorInfo::new(file.to_owned(), err)))
                    }
                }
            });

            match result {
                Ok(_) => {
                    finished_atomic.store(true, Ordering::Relaxed);
                    *file_infos.lock().unwrap() = local_file_infos;
                }
                Err(err) => {
                    // *state_w.write().unwrap() = Some(AnalyzeState::Err(err.into()));
                }
            }
        });

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
