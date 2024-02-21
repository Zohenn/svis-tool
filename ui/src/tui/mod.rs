mod core;
mod widget_utils;
mod widgets;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::*,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use crate::theme;

use self::{
    core::{
        custom_widget::{CustomWidget, RenderContext},
        FocusableWidgetState, HandleEventResult,
    },
    widgets::file_list::{render_file_list, AnalyzeState, FileListState},
    widgets::{dialog::DialogContent, fps::FpsWidget, mapping_info::FileInfoState, path_input::PathInputWidget},
    widgets::{
        path_input::{render_path_input, PathState},
        search_dialog::SearchDialogState,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FocusableWidget {
    PathInput,
    FileList,
    FileInfo,
    SearchDialog,
}

pub struct App {
    focused_widget: Option<FocusableWidget>,
    path_state: PathState,
    file_list_state: FileListState,
    file_info_state: FileInfoState,
    fps: FpsWidget,
    search_dialog: SearchDialogState,
}

impl<'a> Default for App {
    fn default() -> App {
        App {
            focused_widget: Some(FocusableWidget::PathInput),
            path_state: PathState::default(),
            file_list_state: FileListState { analyze_state: None },
            file_info_state: FileInfoState::default(),
            fps: FpsWidget::default(),
            search_dialog: SearchDialogState::default(),
        }
    }
}

impl App {
    fn focused_widget_state(&mut self) -> Option<&mut dyn FocusableWidgetState> {
        match self.focused_widget {
            Some(FocusableWidget::PathInput) => Some(&mut self.path_state),
            Some(FocusableWidget::FileList) => Some(&mut self.file_list_state),
            Some(FocusableWidget::FileInfo) => Some(&mut self.file_info_state),
            Some(FocusableWidget::SearchDialog) => Some(&mut self.search_dialog),
            None => None,
        }
    }

    fn handle_event_result(&mut self, result: HandleEventResult) {
        match result {
            HandleEventResult::Blur => self.focused_widget = None,
            HandleEventResult::KeepFocus => {}
            HandleEventResult::ChangeFocus(new_widget) => self.focused_widget = Some(new_widget),
            HandleEventResult::Callback(callback) => {
                let result = callback(self);
                self.handle_event_result(result);
            }
        }
    }
}

pub fn run_tui_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App, initial_path: Option<&str>) -> Result<()> {
    app.path_state.path_input = app
        .path_state
        .path_input
        .with_value(initial_path.unwrap_or("./test_files/work").into());

    match initial_path {
        Some(path) => {
            app.file_list_state.analyze_path(path.into());
            app.focused_widget = Some(FocusableWidget::FileList);
        }
        _ => {}
    }

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // event::read is blocking, event::poll is not
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('f')) {
                    app.fps.toggle();
                } else {
                    let previously_focused_widget = app.focused_widget;

                    match app.focused_widget_state() {
                        Some(widget_state) => {
                            let result = widget_state.handle_events(key);
                            app.handle_event_result(result);

                            if app.focused_widget.is_some() && previously_focused_widget != app.focused_widget {
                                app.focused_widget_state().unwrap().on_focus();
                            }
                        }
                        None => match key.code {
                            KeyCode::Char('p') => {
                                app.focused_widget = Some(FocusableWidget::PathInput);
                            }
                            KeyCode::Char('f') => {
                                app.focused_widget = Some(FocusableWidget::FileList);
                                match &mut app.file_list_state.analyze_state {
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
}

fn ui(f: &mut Frame, app: &mut App) {
    f.render_widget(Block::new().bg(theme::BACKGROUND), f.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(f.size());

    render_help_message(f, app, chunks[0]);

    let context = RenderContext::new(app, f, FocusableWidget::PathInput);
    PathInputWidget.render(context, chunks[1]);
    // app.path_state.render(context, chunks[1]);
    // render_path_input(f, app, chunks[1]);

    render_file_list(f, app, chunks[2]);

    f.render_widget(
        &mut app.fps,
        Layout::horizontal([Constraint::Min(0), Constraint::Length(10)]).areas::<2>(chunks[0])[1],
    );

    app.search_dialog.render_dialog(
        f,
        f.size(),
        matches!(app.focused_widget, Some(FocusableWidget::SearchDialog)),
    );
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
    let text = Text::from(Line::from(msg)).patch_style(style);

    f.render_widget(Paragraph::new(text), rect);
}
