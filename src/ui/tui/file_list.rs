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
    pub file_infos: StatefulList<SourceMappingInfo>,
    pub files_with_errors: Vec<(String, Error)>,
}

pub fn render_file_list(f: &mut Frame, app: &mut App, rect: Rect) {
    match &mut *app.file_list_state.analyze_state.write().unwrap() {
        Some(AnalyzeState::Pending(files_checked)) => {
            centered_text(f, &format!("Files checked: {files_checked}"), rect);
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
                    let content = vec![Line::from(vec![
                        info.source_mapping.file().into(),
                        " ".into(),
                        format_bytes(info.source_mapping.source_file_without_source_map_len()).highlight(),
                    ])];
                    ListItem::new(content)
                })
                .collect();

            let label = Line::from(vec!["f".key().into(), "ile list".into()]);
            let mut block = default_block().title(label);

            if let Some(item) = selected_item {
                render_mapping_info(f, &mut app.file_info_state, item, chunks[1]);

                block = block.title(
                    Title::from(Line::from(vec![" ↑↓ jk".key().into(), " select".into()])).position(Position::Bottom),
                );
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

pub fn render_mapping_info(f: &mut Frame, file_info_state: &mut FileInfoState, info: &SourceMappingInfo, rect: Rect) {
    let mapping = &info.source_mapping;

    let text: Text = if mapping.is_empty() {
        vec!["File contains empty sourcemap (both \"sources\" and \"mappings\" arrays are empty).".into()].into()
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

    let height = calculate_height(&text, rect.width);

    file_info_state.max_height = rect.height;
    file_info_state.text_height = height;

    f.render_widget(
        Paragraph::new(text)
            .block(default_block().title(format!("{}/{}", height, rect.height)))
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
    let max_line_width = max_line_width as f32;

    for line in &text.lines {
        sum += (line.width() as f32 / max_line_width).ceil() as u16;
    }

    sum
}
