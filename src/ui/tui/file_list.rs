use std::{
    cmp::Ordering,
    sync::{Arc, RwLock},
};

use anyhow::Error;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, List, ListItem, Paragraph, Widget, Wrap,
    },
    Frame,
};

use crate::{
    core::analyzer::{SourceMappingFileInfo, SourceMappingInfo},
    ui::utils::{format_bytes, format_percentage, without_relative_part},
};

use super::{
    core::{FocusableWidgetState, HandleEventResult, SortOrder, StatefulList},
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

impl<'a> FocusableWidgetState for FileListState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match &mut *self.analyze_state.write().unwrap() {
            Some(AnalyzeState::Done(state)) => match event.code {
                KeyCode::Esc => {
                    state.file_infos.unselect();
                    return HandleEventResult::Blur;
                }
                KeyCode::Down | KeyCode::Char('j') => state.file_infos.next(),
                KeyCode::Up | KeyCode::Char('k') => state.file_infos.previous(),
                KeyCode::Char('s') => {
                    state.sort(FileInfoSort::Size);
                }
                KeyCode::Char('n') => {
                    state.sort(FileInfoSort::Name);
                }
                // KeyCode::Char('s') => state.file_infos.sort(|a, b| match (a, b) {
                //     (FileInfoType::Info(a), FileInfoType::Info(b)) => b
                //         .source_mapping
                //         .source_file_without_source_map_len()
                //         .cmp(&a.source_mapping.source_file_without_source_map_len()),
                //     (FileInfoType::Info(_), FileInfoType::Err(_)) => Ordering::Less,
                //     (FileInfoType::Err(_), FileInfoType::Info(_)) => Ordering::Greater,
                //     (FileInfoType::Err(_), FileInfoType::Err(_)) => Ordering::Equal,
                // }),
                // KeyCode::Char('n') => state.file_infos.sort(|a, b| {
                //     let values = [a, b].map(|val| match val {
                //         FileInfoType::Info(v) => &v.source_mapping.file_name,
                //         FileInfoType::Err(v) => &v.file_name,
                //     });
                //
                //     values[0].cmp(values[1])
                // }),
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
    pub sort: FileInfoSort,
    pub sort_order: SortOrder,
}

impl AnalyzeDoneState {
    pub fn new(files_checked: u16, file_infos: Vec<FileInfoType>) -> Self {
        AnalyzeDoneState {
            files_checked,
            file_infos: StatefulList::with_items(file_infos),
            sort: FileInfoSort::Name,
            sort_order: SortOrder::Asc,
        }
    }

    pub fn sort(&mut self, sort: FileInfoSort) {
        let sort_order = if &self.sort == &sort {
            self.sort_order.reverse()
        } else {
            SortOrder::Asc
        };

        self.sort = sort;
        self.sort_order = sort_order;

        let sort_function = match self.sort {
            FileInfoSort::Size => Self::sort_by_size,
            FileInfoSort::Name => Self::sort_by_name,
        };

        self.file_infos.sort(sort_function, self.sort_order);
    }

    fn sort_by_size(a: &FileInfoType, b: &FileInfoType) -> Ordering {
        match (a, b) {
            (FileInfoType::Info(a), FileInfoType::Info(b)) => b
                .source_mapping
                .source_file_without_source_map_len()
                .cmp(&a.source_mapping.source_file_without_source_map_len()),
            (FileInfoType::Info(_), FileInfoType::Err(_)) => Ordering::Less,
            (FileInfoType::Err(_), FileInfoType::Info(_)) => Ordering::Greater,
            (FileInfoType::Err(_), FileInfoType::Err(_)) => Ordering::Equal,
        }
    }

    fn sort_by_name(a: &FileInfoType, b: &FileInfoType) -> Ordering {
        let values = [a, b].map(|val| match val {
            FileInfoType::Info(v) => &v.source_mapping.file_name,
            FileInfoType::Err(v) => &v.file_name,
        });

        values[0].cmp(values[1])
    }
}

pub enum FileInfoType {
    Info(SourceMappingInfo),
    Err(SourceMappingErrorInfo),
}

pub struct SourceMappingErrorInfo {
    file: String,
    error: Error,
    file_name: String,
}

impl SourceMappingErrorInfo {
    pub fn new(file: String, error: Error) -> Self {
        let file_name = match file.rfind('/') {
            Some(pos) => file.get((pos + 1)..).unwrap_or(&file),
            None => &file,
        }
        .to_string();

        SourceMappingErrorInfo { file, error, file_name }
    }
}

#[derive(PartialEq)]
pub enum FileInfoSort {
    Size,
    Name,
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
                    let file_name = match info {
                        FileInfoType::Info(info) => &info.source_mapping.file_name,
                        FileInfoType::Err(error_info) => &error_info.file_name,
                    };
                    let mut content = vec!["./".into(), file_name.into(), " ".into()];

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
                    Title::from(Line::from(vec![
                        " ↑↓ jk".key().into(),
                        " select ".white().into(),
                        "|".dark_gray().into(),
                        " sort: ".white().into(),
                        "s".key().into(),
                        "ize, ".white().into(),
                        "n".key().into(),
                        "ame ".white().into(),
                    ]))
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
        FileInfoType::Err(error_info) => error_info.error.to_string().into(),
    };

    let mut block = default_block();
    if is_focused {
        block = block.border_style(Style::default().yellow());
    }

    let block_inner = block.inner(rect);

    let height = calculate_height(&text, block.clone(), rect);

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
        let max_scroll = self.text_height.saturating_sub(self.max_height);
        if max_scroll > 0 {
            match event.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.scroll == max_scroll {
                        self.scroll = 0;
                    } else {
                        self.scroll += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.scroll == 0 {
                        self.scroll = max_scroll;
                    } else {
                        self.scroll -= 1;
                    }
                }
                _ => {}
            }
        } else {
            self.scroll = 0;
        }

        if matches!(event.code, KeyCode::Esc) {
            HandleEventResult::ChangeFocus(FocusableWidget::FileList)
        } else {
            HandleEventResult::KeepFocus
        }
    }
}

fn calculate_height<'a>(text: &Text, block: Block, area: Rect) -> u16 {
    let area = Rect::new(area.x, area.y, area.width, u16::MAX / area.width);
    let mut buffer = Buffer::empty(area);

    let paragraph = Paragraph::new(text.clone())
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((0, 0));

    paragraph.render(area, &mut buffer);

    for y in buffer.area.top()..buffer.area.bottom() {
        let x = buffer.area.left() + 1;

        if buffer.get(x, y).symbol == " " {
            return y - 1 - area.y;
        }
    }

    0
}
