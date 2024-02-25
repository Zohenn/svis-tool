use std::{
    cmp::Ordering as CmpOrdering,
    fmt::Debug,
    sync::{
        atomic::{AtomicU16, AtomicU8, Ordering},
        mpsc, Arc, Mutex,
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
};
use threadpool::Builder as ThreadPoolBuilder;

use core::{analyzer::SourceMappingInfo, discover_files, handle_file};

use crate::{
    keybindings,
    tui::core::custom_widget::{CustomWidget, RenderContext},
    utils::format_bytes,
};

use crate::tui::{
    core::{FocusableWidgetState, HandleEventResult, SortOrder, StatefulList},
    widget_utils::{centered_text, default_block, CustomStyles},
    widgets::mapping_info::FileInfoState,
    App, FocusableWidget,
};

use super::mapping_info::MappingInfoWidget;

pub enum AnalyzeState {
    Pending(AnalyzePendingState),
    Done(AnalyzeDoneState),
    Err(Box<anyhow::Error>),
}

pub struct FileListState {
    pub analyze_state: Option<AnalyzeState>,
}

impl FileListState {
    pub fn analyze_path(&mut self, path: String) {
        let pending_state = AnalyzePendingState::default();
        let files_checked_atomic = pending_state.count.clone();
        let file_infos = pending_state.file_infos.clone();
        let state_atomic = pending_state.state.clone();
        let error = pending_state.error.clone();
        self.analyze_state = Some(AnalyzeState::Pending(pending_state));

        std::thread::spawn(move || {
            let files_to_check = match discover_files(&path) {
                Ok(files_to_check) => files_to_check,
                Err(err) => {
                    *error.lock().unwrap() = err.into();
                    state_atomic.store(OperationState::Err as u8, Ordering::Relaxed);
                    return;
                }
            };

            let thread_pool = ThreadPoolBuilder::new().build();

            let (sender, receiver) = mpsc::channel::<FileInfoType>();

            for file in files_to_check {
                let sender = sender.clone();
                let files_checked_atomic = files_checked_atomic.clone();

                thread_pool.execute(move || {
                    let file_info = match handle_file(&file) {
                        Ok(info) => FileInfoType::Info(info),
                        Err(err) => FileInfoType::Err(SourceMappingErrorInfo::new(file.to_owned(), err)),
                    };

                    sender.send(file_info).unwrap();
                    files_checked_atomic.fetch_add(1, Ordering::Relaxed);
                });
            }

            drop(sender);

            *file_infos.lock().unwrap() = receiver.iter().collect::<Vec<_>>();
            state_atomic.store(OperationState::Done as u8, Ordering::Relaxed);
        });
    }
}

impl FocusableWidgetState for FileListState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        if let Some(AnalyzeState::Done(state)) = &mut self.analyze_state {
            match event.code {
                KeyCode::Esc => {
                    state.file_infos.unselect();
                    return HandleEventResult::Blur;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.file_infos.next();
                    return HandleEventResult::Callback(Box::new(Self::callback));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.file_infos.previous();
                    return HandleEventResult::Callback(Box::new(Self::callback));
                }
                KeyCode::Char('s') => {
                    state.sort(FileInfoSort::Size);
                    return HandleEventResult::Callback(Box::new(Self::callback));
                }
                KeyCode::Char('n') => {
                    state.sort(FileInfoSort::Name);
                    return HandleEventResult::Callback(Box::new(Self::callback));
                }
                KeyCode::Char('f') => return HandleEventResult::ChangeFocus(FocusableWidget::SearchDialog),
                KeyCode::Enter => return HandleEventResult::ChangeFocus(FocusableWidget::FileInfo),
                _ => {}
            }
        }

        if matches!(event.code, KeyCode::Esc) {
            HandleEventResult::Blur
        } else {
            HandleEventResult::KeepFocus
        }
    }

    fn callback(app: &mut App) -> HandleEventResult {
        app.file_info_state = FileInfoState::default();
        HandleEventResult::KeepFocus
    }
}

pub enum OperationState {
    Pending,
    Done,
    Err,
}

pub struct AnalyzePendingState {
    pub count: Arc<AtomicU16>,
    pub state: Arc<AtomicU8>,
    pub error: Arc<Mutex<Box<anyhow::Error>>>,
    pub file_infos: Arc<Mutex<Vec<FileInfoType>>>,
}

impl Default for AnalyzePendingState {
    fn default() -> Self {
        AnalyzePendingState {
            count: Arc::default(),
            state: Arc::default(),
            error: Arc::new(Mutex::new(Box::new(anyhow::anyhow!("")))),
            file_infos: Arc::default(),
        }
    }
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
        let sort_order = if self.sort == sort {
            self.sort_order.reverse()
        } else {
            SortOrder::Asc
        };

        self.sort_with_order(sort, sort_order)
    }

    pub fn sort_with_order(&mut self, sort: FileInfoSort, sort_order: SortOrder) {
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
                .actual_source_file_len()
                .cmp(&b.source_mapping.actual_source_file_len()),
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

#[derive(Clone, Copy, PartialEq)]
pub enum FileInfoSort {
    Size,
    Name,
}

pub struct FileListWidget;

impl CustomWidget for FileListWidget {
    fn bound_state(&self) -> Option<FocusableWidget> {
        Some(FocusableWidget::FileList)
    }

    fn render<'widget, 'app: 'widget>(&self, mut context: RenderContext<'app, '_>, rect: Rect) {
        let is_focused = context.is_focused();

        // Looks kinda funny, but allows for mutex value to be moved out of struct.
        let mut analyze_state = context.app_mut().file_list_state.analyze_state.take();

        match analyze_state {
            Some(AnalyzeState::Pending(pending_state)) => {
                let files_checked = pending_state.count.load(Ordering::Relaxed);
                centered_text(context.frame_mut(), &format!("Files checked: {}", files_checked), rect);

                match pending_state.get_state() {
                    OperationState::Done => {
                        let file_infos = Arc::try_unwrap(pending_state.file_infos).unwrap().into_inner().unwrap();
                        let mut done_state = AnalyzeDoneState::new(files_checked, file_infos);
                        done_state.file_infos.next();
                        done_state.sort_with_order(done_state.sort, done_state.sort_order);
                        analyze_state = Some(AnalyzeState::Done(done_state));
                    }
                    OperationState::Pending => {
                        analyze_state = Some(AnalyzeState::Pending(pending_state));
                    }
                    OperationState::Err => {
                        let error = Arc::try_unwrap(pending_state.error).unwrap().into_inner().unwrap();
                        analyze_state = Some(AnalyzeState::Err(error));
                    }
                }
            }
            Some(AnalyzeState::Err(ref err)) => {
                centered_text(context.frame_mut(), &err.to_string(), rect);
            }
            Some(AnalyzeState::Done(ref mut state)) => {
                let has_selection = state.file_infos.has_selection();

                let constraints = match has_selection {
                    true => [Constraint::Percentage(50), Constraint::Percentage(50)],
                    false => [Constraint::Percentage(100), Constraint::Percentage(0)],
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
                            content.push(format_bytes(info.source_mapping.actual_source_file_len()).highlight());
                            content.extend([
                                " (".into(),
                                info.info_by_file.len().to_string().highlight2(),
                                " files".highlight2(),
                                ")".into(),
                            ]);
                        } else {
                            content.push("!".error());
                        }
                        ListItem::new(Line::from(content))
                    })
                    .collect();

                let label = Line::from(keybindings!("f""ile list"));
                let mut block = default_block().title(label);

                if has_selection {
                    let title_contents = keybindings!(
                        "↑↓ jk"" select ";
                        "|".dark_gray(),
                        " sort: ".white();,
                        "s""ize, ", "n""ame ";
                        "| ".dark_gray();,
                        "f""ind source file"
                    );

                    block = block
                        .title(Title::from(Line::from(title_contents)).position(Position::Bottom))
                        .title(
                            Title::from(Line::from(
                                format!(
                                    " {}/{} ",
                                    state.file_infos.state.selected().unwrap() + 1,
                                    state.file_infos.items.len()
                                )
                                .white(),
                            ))
                            .position(Position::Bottom)
                            .alignment(Alignment::Right),
                        );
                }

                if is_focused {
                    block = block.border_style(Style::default().yellow());
                }

                let (app, frame) = context.app_frame_mut();

                let file_infos_list = List::new(file_infos)
                    .block(block)
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
                frame.render_stateful_widget(file_infos_list, chunks[0], &mut state.file_infos.state);

                if let Some(item) = state.file_infos.selected_item() {
                    let context = RenderContext::new(app, frame, Some(FocusableWidget::FileInfo));
                    MappingInfoWidget { info: item }.render(context, chunks[1]);
                }
            }
            None => {
                centered_text(context.frame_mut(), "Enter path to start", rect);
            }
        }

        context.app_mut().file_list_state.analyze_state = analyze_state;
    }
}
