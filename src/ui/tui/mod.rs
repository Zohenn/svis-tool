use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use anyhow::{Error, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Rect},
    style::*,
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
    Frame, Terminal,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::core::{analyze_path, analyzer::SourceMappingInfo};

use super::utils::format_bytes;

struct PathState {
    path_input: Input,
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

enum HandleEventResult {
    Blur,
    KeepFocus,
    Callback(fn(&mut App) -> HandleEventResult), //Box<dyn FnMut(&mut App) -> HandleEventResult>),
}

trait FocusableWidgetState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult;

    fn callback(_app: &mut App) -> HandleEventResult
    where
        Self: Sized,
    {
        HandleEventResult::KeepFocus
    }
}

#[derive(Clone, Copy)]
enum FocusableWidget {
    PathInput,
    FileList,
}

pub struct App {
    focused_widget: Option<FocusableWidget>,
    path_state: PathState,
    file_list_state: FileListState,
}

impl Default for App {
    fn default() -> App {
        App {
            focused_widget: None,
            path_state: PathState::default(),
            file_list_state: FileListState {
                analyze_state: Arc::new(RwLock::new(None)),
            },
        }
    }
}

enum AnalyzeState {
    Pending(u16),
    Done(AnalyzeDoneState),
}

struct FileListState {
    analyze_state: Arc<RwLock<Option<AnalyzeState>>>,
}

impl FocusableWidgetState for FileListState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match &mut *self.analyze_state.write().unwrap() {
            Some(AnalyzeState::Done(state)) => match event.code {
                KeyCode::Esc => {
                    state.file_infos.unselect();
                    return HandleEventResult::Blur;
                }
                KeyCode::Down => state.file_infos.next(),
                KeyCode::Up => state.file_infos.previous(),
                _ => {}
            },
            _ => {}
        }

        if matches!(event.code, KeyCode::Esc) {
            HandleEventResult::Blur
        } else {
            HandleEventResult::KeepFocus
        }
    }
}

struct AnalyzeDoneState {
    files_checked: u16,
    file_infos: StatefulList<SourceMappingInfo>,
    files_with_errors: Vec<(String, Error)>,
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub fn run_tui_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    app.path_state.path_input = app
        .path_state
        .path_input
        .with_value("/var/www/hrappka-frontend/dist/spa/assets/".into());
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // event::read is blocking, event::poll is not
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.focused_widget {
                    Some(widget) => {
                        let widget_state: &mut dyn FocusableWidgetState = match widget {
                            FocusableWidget::PathInput => &mut app.path_state,
                            FocusableWidget::FileList => &mut app.file_list_state,
                        };

                        match widget_state.handle_events(key) {
                            HandleEventResult::Blur => app.focused_widget = None,
                            HandleEventResult::KeepFocus => {}
                            HandleEventResult::Callback(callback) => match callback(&mut app) {
                                HandleEventResult::Blur => app.focused_widget = None,
                                HandleEventResult::KeepFocus => {}
                                HandleEventResult::Callback(_) => unreachable!(),
                            },
                        }
                    }
                    None => match key.code {
                        KeyCode::Char('p') => {
                            app.focused_widget = Some(FocusableWidget::PathInput);
                        }
                        KeyCode::Char('l') => {
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

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(f.size());

    help_message(f, app, chunks[0]);

    path_input(f, app, chunks[1]);

    file_list(f, app, chunks[2]);
}

fn help_message(f: &mut Frame, app: &App, rect: Rect) {
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

fn path_input(f: &mut Frame, app: &App, rect: Rect) {
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

fn file_list(f: &mut Frame, app: &App, rect: Rect) {
    match &mut *app.file_list_state.analyze_state.write().unwrap() {
        Some(AnalyzeState::Pending(files_checked)) => {
            centered_text(f, &format!("Files checked: {files_checked}"), rect);
        }
        Some(AnalyzeState::Done(state)) => {
            let messages: Vec<ListItem> = state
                .file_infos
                .items
                .iter()
                .map(|info| {
                    let content = vec![Line::from(vec![
                        info.source_mapping.file().into(),
                        " ".into(),
                        format_bytes(info.source_mapping.source_file_without_source_map_len())
                            .bold()
                            .cyan(),
                    ])];
                    ListItem::new(content)
                })
                .collect();

            let label = Line::from(vec!["File list (".into(), "l".underlined(), ")".into()]);

            let messages = List::new(messages)
                .block(Block::default().borders(Borders::ALL).title(label))
                .highlight_style(Style::default().bg(Color::LightGreen).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(messages, rect, &mut state.file_infos.state);
        }
        None => {
            centered_text(f, "Enter path to start", rect);
        }
    }
}

fn centered_text(f: &mut Frame, text: &str, rect: Rect) {
    f.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::new(0, 0, rect.height / 2, 0)),
            )
            .alignment(Alignment::Center),
        rect,
    );
}
