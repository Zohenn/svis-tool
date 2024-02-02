use std::{
    cmp::Ordering as CmpOrdering,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, AtomicU16, AtomicU8, Ordering},
        Arc, Mutex, RwLock,
    },
};

use anyhow::Error;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    text::Line,
    widgets::{
        block::{Position, Title},
        List, ListItem,
    },
    Frame,
};

use core::analyzer::SourceMappingInfo;

use crate::{keybindings, utils::format_bytes};

use super::{
    core::{FocusableWidgetState, HandleEventResult, SortOrder, StatefulList},
    mapping_info::{render_mapping_info, FileInfoState},
    widget_utils::{centered_text, default_block, CustomStyles},
    App, FocusableWidget,
};

pub enum AnalyzeState {
    Pending(AnalyzePendingState),
    Done(AnalyzeDoneState),
    Err(Box<anyhow::Error>),
}

pub struct FileListState {
    pub analyze_state: Option<AnalyzeState>,
}

impl<'a> FocusableWidgetState for FileListState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match &mut self.analyze_state {
            Some(AnalyzeState::Done(state)) => match event.code {
                KeyCode::Esc => {
                    state.file_infos.unselect();
                    return HandleEventResult::Blur;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.file_infos.next();
                    return HandleEventResult::Callback(|app| {
                        app.file_info_state = FileInfoState::default();
                        HandleEventResult::KeepFocus
                    });
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.file_infos.previous();
                    return HandleEventResult::Callback(|app| {
                        app.file_info_state = FileInfoState::default();
                        HandleEventResult::KeepFocus
                    });
                }
                KeyCode::Char('s') => {
                    state.sort(FileInfoSort::Size);
                }
                KeyCode::Char('n') => {
                    state.sort(FileInfoSort::Name);
                }
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

pub enum OperationState {
    Pending,
    Done,
    Err,
}

#[derive(Default)]
pub struct AnalyzePendingState {
    pub count: Arc<AtomicU16>,
    pub state: Arc<AtomicU8>,
    pub error: Arc<Mutex<Option<Box<anyhow::Error>>>>,
    pub file_infos: Arc<Mutex<Vec<FileInfoType>>>,
}

impl AnalyzePendingState {
    pub fn get_state(&self) -> OperationState {
        match self.state.load(Ordering::Relaxed) {
            0 => OperationState::Pending,
            1 => OperationState::Done,
            2 => OperationState::Err,
            _ => unreachable!(),
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

    fn sort_by_size(a: &FileInfoType, b: &FileInfoType) -> CmpOrdering {
        match (a, b) {
            (FileInfoType::Info(a), FileInfoType::Info(b)) => a
                .source_mapping
                .source_file_without_source_map_len()
                .cmp(&b.source_mapping.source_file_without_source_map_len()),
            (FileInfoType::Info(_), FileInfoType::Err(_)) => CmpOrdering::Greater,
            (FileInfoType::Err(_), FileInfoType::Info(_)) => CmpOrdering::Less,
            (FileInfoType::Err(_), FileInfoType::Err(_)) => CmpOrdering::Equal,
        }
    }

    fn sort_by_name(a: &FileInfoType, b: &FileInfoType) -> CmpOrdering {
        let values = [a, b].map(|val| match val {
            FileInfoType::Info(v) => &v.source_mapping.file_name,
            FileInfoType::Err(v) => &v.file_name,
        });

        values[0].cmp(values[1])
    }
}

#[derive(Debug)]
pub enum FileInfoType {
    Info(SourceMappingInfo),
    Err(SourceMappingErrorInfo),
}

#[derive(Debug)]
pub struct SourceMappingErrorInfo {
    pub file: String,
    pub error: Error,
    pub file_name: String,
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

    // Looks kinda funny, but allows for mutex value to be moved out of struct.
    let mut analyze_state = std::mem::replace(&mut app.file_list_state.analyze_state, None);

    match analyze_state {
        Some(AnalyzeState::Pending(pending_state)) => {
            let files_checked = pending_state.count.load(Ordering::Relaxed);
            centered_text(f, &format!("Files checked: {}", files_checked), rect);

            match pending_state.get_state() {
                OperationState::Done => {
                    let file_infos = Arc::try_unwrap(pending_state.file_infos).unwrap().into_inner().unwrap();
                    let mut done_state = AnalyzeDoneState::new(files_checked, file_infos);
                    done_state.file_infos.next();
                    analyze_state = Some(AnalyzeState::Done(done_state));
                }
                OperationState::Pending => {
                    analyze_state = Some(AnalyzeState::Pending(pending_state));
                }
                OperationState::Err => {
                    let error = Arc::try_unwrap(pending_state.error)
                        .unwrap()
                        .into_inner()
                        .unwrap()
                        .unwrap();
                    analyze_state = Some(AnalyzeState::Err(error));
                }
            }
        }
        Some(AnalyzeState::Err(ref err)) => {
            centered_text(f, &err.to_string(), rect);
        }
        Some(AnalyzeState::Done(ref mut state)) => {
            let selected_item = state.file_infos.selected_item();

            let constraints = match selected_item {
                Some(_) => [Constraint::Percentage(50), Constraint::Percentage(50)],
                None => [Constraint::Percentage(100), Constraint::Percentage(0)],
            };

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints.as_ref())
                .split(rect);

            let file_infos: Vec<ListItem> = state
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
                        content.extend(
                            [
                                " (".into(),
                                info.info_by_file.len().to_string().highlight2(),
                                " files".highlight2(),
                                ")".into(),
                            ]
                            .into_iter(),
                        );
                    } else {
                        content.push("!".error());
                    }
                    ListItem::new(Line::from(content))
                })
                .collect();

            let label = Line::from(keybindings!("f""ile list"));
            let mut block = default_block().title(label);

            if let Some(item) = selected_item {
                let is_focused = matches!(app.focused_widget, Some(FocusableWidget::FileInfo));
                render_mapping_info(f, &mut app.file_info_state, item, is_focused, chunks[1]);

                let title_contents = keybindings!(
                    "↑↓ jk"" select ";
                    "|".dark_gray().into(),
                    " sort: ".white().into();,
                    "s""ize, ", "n""ame"
                );

                block = block.title(Title::from(Line::from(title_contents)).position(Position::Bottom));
            }

            if is_focused {
                block = block.border_style(Style::default().yellow());
            }

            let file_infos_list = List::new(file_infos)
                .block(block)
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
            f.render_stateful_widget(file_infos_list, chunks[0], &mut state.file_infos.state);
        }
        None => {
            centered_text(f, "Enter path to start", rect);
        }
    }

    app.file_list_state.analyze_state = analyze_state;
}
