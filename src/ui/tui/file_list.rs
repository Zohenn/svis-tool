use std::sync::{Arc, RwLock};

use anyhow::Error;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        List, ListItem, Paragraph, Wrap,
    },
    Frame,
};

use crate::{
    core::analyzer::{SourceMappingFileInfo, SourceMappingInfo},
    ui::utils::{format_bytes, format_percentage, without_relative_part},
};

use super::{
    core::{FocusableWidgetState, HandleEventResult, StatefulList},
    widget_utils::{centered_text, default_block, CustomStyles},
    App, FocusableWidget,
};

pub enum AnalyzeState {
    Pending(u16),
    Done(AnalyzeDoneState),
    Err(Box<anyhow::Error>),
}

pub struct FileListState {
    pub analyze_state: Arc<RwLock<Option<AnalyzeState>>>,
}

impl FocusableWidgetState for FileListState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match &mut *self.analyze_state.write().unwrap() {
            Some(AnalyzeState::Done(state)) => match event.code {
                KeyCode::Esc => {
                    state.file_infos.unselect();
                    return HandleEventResult::Blur;
                }
                KeyCode::Down | KeyCode::Char('j') => state.file_infos.next(),
                KeyCode::Up | KeyCode::Char('k') => state.file_infos.previous(),
                KeyCode::Enter => return HandleEventResult::ChangeFocus(FocusableWidget::FileInfo),
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

pub struct AnalyzeDoneState {
    pub files_checked: u16,
    pub file_infos: StatefulList<FileInfoType>,
}

pub enum FileInfoType {
    Info(SourceMappingInfo),
    Err((String, Error)),
}

pub fn render_file_list(f: &mut Frame, app: &mut App, rect: Rect) {
    let is_focused = matches!(app.focused_widget, Some(FocusableWidget::FileList));

    match &mut *app.file_list_state.analyze_state.write().unwrap() {
        Some(AnalyzeState::Pending(files_checked)) => {
            centered_text(f, &format!("Files checked: {files_checked}"), rect);
        }
        Some(AnalyzeState::Err(err)) => {
            centered_text(f, &err.to_string(), rect);
        }
        Some(AnalyzeState::Done(state)) => {
            let selected_item = state.file_infos.selected_item();

            let constraints = match selected_item {
                Some(_) => [Constraint::Percentage(50), Constraint::Percentage(50)],
                None => [Constraint::Percentage(100), Constraint::Percentage(0)],
            };

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints.as_ref())
                .split(rect);

            let messages: Vec<ListItem> = state
                .file_infos
                .items
                .iter()
                .map(|info| {
                    let source_path_len = app.path_state.path_input.value().len();
                    let path = match info {
                        FileInfoType::Info(info) => info.source_mapping.file(),
                        FileInfoType::Err((file, _)) => file,
                    };
                    let mut content = vec![
                        "./".into(),
                        (path[source_path_len..].trim_start_matches('/')).into(),
                        " ".into(),
                    ];

                    if let FileInfoType::Info(info) = info {
                        content
                            .push(format_bytes(info.source_mapping.source_file_without_source_map_len()).highlight());
                    } else {
                        content.push("!".error());
                    }
                    ListItem::new(Line::from(content))
                })
                .collect();

            let label = Line::from(vec![" f".key().into(), "ile list ".white().into()]);
            let mut block = default_block().title(label);

            if let Some(item) = selected_item {
                let is_focused = matches!(app.focused_widget, Some(FocusableWidget::FileInfo));
                render_mapping_info(f, &mut app.file_info_state, item, is_focused, chunks[1]);

                block = block.title(
                    Title::from(Line::from(vec![" ↑↓ jk".key().into(), " select ".white().into()]))
                        .position(Position::Bottom),
                );
            }

            if is_focused {
                block = block.border_style(Style::default().yellow());
            }

            let messages = List::new(messages)
                .block(block)
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(messages, chunks[0], &mut state.file_infos.state);
        }
        None => {
            centered_text(f, "Enter path to start", rect);
        }
    }
}

pub fn render_mapping_info(
    f: &mut Frame,
    file_info_state: &mut FileInfoState,
    info: &FileInfoType,
    is_focused: bool,
    rect: Rect,
) {
    let text = match info {
        FileInfoType::Info(info) => {
            let mapping = &info.source_mapping;

            let text: Text = if mapping.is_empty() {
                vec!["File contains empty sourcemap (both \"sources\" and \"mappings\" arrays are empty).".into()]
                    .into()
            } else {
                let sources_root = mapping.get_sources_root();

                let source_file_len = mapping.source_file_without_source_map_len();

                let mut lines = vec![
                    Line::from(vec![
                        "File size: ".into(),
                        format_bytes(source_file_len).highlight().into(),
                        ".".into(),
                    ]),
                    Line::from(vec![
                        "Size contribution per file (all paths are relative to ".into(),
                        sources_root.bold().into(),
                        "):".into(),
                    ]),
                ];

                let mut info_by_file = info.info_by_file.iter().collect::<Vec<&SourceMappingFileInfo>>();
                info_by_file.sort_by_key(|i| i.bytes);

                for file_info in info_by_file.iter().rev() {
                    lines.push(
                        vec![
                            "- ".into(),
                            without_relative_part(info.get_file_name(file_info.file)).bold().into(),
                            ", size ".into(),
                            format_bytes(file_info.bytes as u64).highlight().into(),
                            " (".into(),
                            format_percentage(file_info.bytes as u64, source_file_len)
                                .highlight2()
                                .into(),
                            ")".into(),
                        ]
                        .into(),
                    );
                }

                let sum_bytes = info.sum_bytes as u64;

                lines.push(
                    vec![
                        "Sum: ".into(),
                        format_bytes(sum_bytes).highlight(),
                        " (".into(),
                        format_percentage(sum_bytes, source_file_len).highlight2().into(),
                        ")".into(),
                    ]
                    .into(),
                );

                let rest = source_file_len - sum_bytes;

                lines.push(
                    vec![
                        "Remaining size taken by preamble, imports, whitespace, comments, etc.: ".into(),
                        format_bytes(rest).highlight().into(),
                        " (".into(),
                        format_percentage(rest, source_file_len).highlight2().into(),
                        ")".into(),
                    ]
                    .into(),
                );

                lines.into()
            };

            text
        }
        FileInfoType::Err((_path, err)) => err.to_string().into(),
    };

    let mut block = default_block();
    if is_focused {
        block = block.border_style(Style::default().yellow());
    }

    let block_inner = block.inner(rect);

    let height = calculate_height(&text, block_inner.width);

    file_info_state.max_height = block_inner.height;
    file_info_state.text_height = height;

    f.render_widget(
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((file_info_state.scroll, 0)),
        rect,
    );
}

pub struct FileInfoState {
    pub scroll: u16,
    pub text_height: u16,
    pub max_height: u16,
}

impl Default for FileInfoState {
    fn default() -> Self {
        Self {
            scroll: 0,
            text_height: 0,
            max_height: 0,
        }
    }
}

impl FocusableWidgetState for FileInfoState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match event.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.scroll == self.text_height {
                    self.scroll = 0;
                } else {
                    self.scroll += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll == 0 {
                    self.scroll = self.text_height;
                } else {
                    self.scroll -= 1;
                }
            }
            _ => {}
        }

        if matches!(event.code, KeyCode::Esc) {
            HandleEventResult::ChangeFocus(FocusableWidget::FileList)
        } else {
            HandleEventResult::KeepFocus
        }
    }
}

fn calculate_height(text: &Text, max_line_width: u16) -> u16 {
    let mut sum = 0;

    for line in &text.lines {
        let line_width = line.width() as u16;
        sum += line_width / max_line_width;
        if line_width % max_line_width > 0 {
            sum += 1;
        }
    }

    sum
}
