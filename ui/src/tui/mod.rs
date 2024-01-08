mod core;
mod file_list;
mod mapping_info;
mod path_input;
mod widget_utils;

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::*,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use self::{
    core::{FocusableWidgetState, HandleEventResult},
    file_list::{render_file_list, AnalyzeState, FileListState},
    mapping_info::FileInfoState,
    path_input::{render_path_input, PathState},
};

#[derive(Clone, Copy)]
pub enum FocusableWidget {
    PathInput,
    FileList,
    FileInfo,
}

pub struct App {
    focused_widget: Option<FocusableWidget>,
    path_state: PathState,
    file_list_state: FileListState,
    file_info_state: FileInfoState,
}

impl<'a> Default for App {
    fn default() -> App {
        App {
            focused_widget: Some(FocusableWidget::PathInput),
            path_state: PathState::default(),
            file_list_state: FileListState {
                analyze_state: Arc::new(RwLock::new(None)),
            },
            file_info_state: FileInfoState::default(),
        }
    }
}

pub fn run_tui_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App, initial_path: Option<&str>) -> Result<()> {
    app.path_state.path_input = app
        .path_state
        .path_input
        .with_value(initial_path.unwrap_or("./test_files/work").into());
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // event::read is blocking, event::poll is not
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.focused_widget {
                    Some(widget) => {
                        let widget_state: &mut dyn FocusableWidgetState = match widget {
                            FocusableWidget::PathInput => &mut app.path_state,
                            FocusableWidget::FileList => &mut app.file_list_state,
                            FocusableWidget::FileInfo => &mut app.file_info_state,
                        };

                        match widget_state.handle_events(key) {
                            HandleEventResult::Blur => app.focused_widget = None,
                            HandleEventResult::KeepFocus => {}
                            HandleEventResult::ChangeFocus(new_widget) => app.focused_widget = Some(new_widget),
                            HandleEventResult::Callback(callback) => match callback(&mut app) {
                                HandleEventResult::Blur => app.focused_widget = None,
                                HandleEventResult::KeepFocus => {}
                                HandleEventResult::ChangeFocus(new_widget) => app.focused_widget = Some(new_widget),
                                HandleEventResult::Callback(_) => unreachable!(),
                            },
                        }
                    }
                    None => match key.code {
                        KeyCode::Char('p') => {
                            app.focused_widget = Some(FocusableWidget::PathInput);
                        }
                        KeyCode::Char('f') => {
                            app.focused_widget = Some(FocusableWidget::FileList);
                            match &mut *app.file_list_state.analyze_state.write().unwrap() {
                                Some(AnalyzeState::Done(state)) => {
                                    state.file_infos.next();
                                }
                                _ => {}
                            };
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    f.render_widget(Block::new().style(Style::default().on_black()), f.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(f.size());

    render_help_message(f, app, chunks[0]);

    render_path_input(f, app, chunks[1]);

    render_file_list(f, app, chunks[2]);
}

fn render_help_message(f: &mut Frame, app: &App, rect: Rect) {
    let (msg, style) = match app.focused_widget {
        Some(_) => (
            vec!["Press ".into(), "Esc".bold(), " to unfocus".into()],
            Style::default(),
        ),
        None => (
            vec!["Press ".into(), "q".bold(), " to close the app".into()],
            Style::default(),
        ),
    };
    let mut text = Text::from(Line::from(msg));
    text.patch_style(style);

    f.render_widget(Paragraph::new(text), rect);
}
