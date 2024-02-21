mod core;
mod widget_utils;
mod widgets;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
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
    widgets::file_list::{AnalyzeState, FileListState},
    widgets::{
        dialog::DialogContent, file_list::FileListWidget, fps::FpsWidget, mapping_info::FileInfoState,
        path_input::PathInputWidget,
    },
    widgets::{path_input::PathState, search_dialog::SearchDialogState},
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

        let event_poll_timeout = if app.fps.visible() { 0 } else { 100 };

        // event::read is blocking, event::poll is not
        if event::poll(Duration::from_millis(event_poll_timeout))? {
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

    let widgets: [Box<dyn CustomWidget>; 3] = [
        Box::new(HelpMessageWidget),
        Box::new(PathInputWidget),
        Box::new(FileListWidget),
    ];

    for (index, widget) in widgets.into_iter().enumerate() {
        let context = RenderContext::new(app, f, widget.bound_state());
        widget.render(context, chunks[index]);
    }

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

struct HelpMessageWidget;

impl CustomWidget for HelpMessageWidget {
    fn bound_state(&self) -> Option<FocusableWidget> {
        None
    }

    fn render<'widget, 'app: 'widget>(&self, mut context: RenderContext<'app, '_>, rect: Rect) {
        let (msg, style) = match context.app().focused_widget {
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
        context.frame_mut().render_widget(Paragraph::new(text), rect);
    }
}
