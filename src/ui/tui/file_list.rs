use std::sync::{Arc, RwLock};

use anyhow::Error;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::Rect,
    style::*,
    text::Line,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::{core::analyzer::SourceMappingInfo, ui::utils::format_bytes};

use super::{
    core::{FocusableWidgetState, HandleEventResult, StatefulList},
    widget_utils::centered_text,
    App,
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

pub struct AnalyzeDoneState {
    pub files_checked: u16,
    pub file_infos: StatefulList<SourceMappingInfo>,
    pub files_with_errors: Vec<(String, Error)>,
}

pub fn render_file_list(f: &mut Frame, app: &App, rect: Rect) {
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
