use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::*,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        *,
    },
};

use core::analyzer::SourceMappingFileInfo;
use std::ops::Add;

use crate::{
    keybindings,
    tui::core::{
        custom_widget::{CustomWidget, RenderContext},
        ListOperations,
    },
    utils::{format_bytes, format_percentage, without_relative_part},
};

use crate::tui::{
    core::{
        tree::{Tree, TreeState},
        FocusableWidgetState, HandleEventResult,
    },
    widget_utils::{default_block, CustomStyles},
    widgets::file_list::FileInfoType,
    FocusableWidget,
};

pub struct MappingInfoWidget<'info> {
    pub info: &'info FileInfoType,
}

impl CustomWidget for MappingInfoWidget<'_> {
    fn bound_state(&self) -> Option<FocusableWidget> {
        Some(FocusableWidget::FileInfo)
    }

    fn render<'widget, 'app: 'widget>(&self, mut context: RenderContext<'app, '_>, rect: Rect) {
        let file_info_state = &mut context.app_mut().file_info_state;

        match file_info_state.view_type {
            FileInfoViewType::Tree if matches!(self.info, FileInfoType::Info(info) if !info.source_mapping.is_empty()) =>
            {
                TreeInfoWidget { info: self.info }.render(context, rect);
            }
            _ => {
                ParagraphInfoWidget { info: self.info }.render(context, rect);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct TreeAggregation {
    bytes: u64,
}

impl Add for TreeAggregation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            bytes: self.bytes + rhs.bytes,
        }
    }
}

struct TreeInfoWidget<'info> {
    info: &'info FileInfoType,
}

impl CustomWidget for TreeInfoWidget<'_> {
    fn bound_state(&self) -> Option<FocusableWidget> {
        Some(FocusableWidget::FileInfo)
    }

    fn render<'widget, 'app: 'widget>(&self, mut context: RenderContext<'app, '_>, rect: Rect) {
        let is_focused = context.is_focused();
        let (app, frame) = context.app_frame_mut();
        let file_info_state = &mut app.file_info_state;

        let FileInfoType::Info(info) = self.info else {
            unreachable!()
        };

        let mapping = &info.source_mapping;
        let source_file_len = mapping.actual_source_file_len();
        let aggregator_source_file_len = source_file_len;

        // TODO: try not to create the tree from scratch on every render
        let tree = Tree::from(info.info_by_file.iter().collect::<Vec<_>>(), |item| {
            without_relative_part(info.get_file_name(item.file))
        })
        .with_aggregator(
            |info| TreeAggregation {
                bytes: info.bytes as u64,
            },
            move |aggregation| {
                vec![
                    format_bytes(aggregation.bytes).highlight(),
                    " (".into(),
                    format_percentage(aggregation.bytes, aggregator_source_file_len).highlight2(),
                    ") ".into(),
                ]
            },
        );

        let list_items = tree.as_list_items(&mut file_info_state.tree_state, |file_info| {
            vec![
                without_relative_part(info.get_file_name(file_info.file))
                    .split('/')
                    .last()
                    .unwrap()
                    .into(),
                " ".into(),
                format_bytes(file_info.bytes as u64).highlight(),
                " (".into(),
                format_percentage(file_info.bytes as u64, source_file_len).highlight2(),
                ")".into(),
            ]
        });

        let block = get_block(is_focused);

        frame.render_stateful_widget(
            List::new(list_items)
                .block(block)
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            rect,
            &mut file_info_state.tree_state.list_state,
        );
    }
}

struct ParagraphInfoWidget<'info> {
    info: &'info FileInfoType,
}

impl CustomWidget for ParagraphInfoWidget<'_> {
    fn bound_state(&self) -> Option<FocusableWidget> {
        Some(FocusableWidget::FileInfo)
    }

    fn render<'widget, 'app: 'widget>(&self, mut context: RenderContext<'app, '_>, rect: Rect) {
        let is_focused = context.is_focused();
        let (app, frame) = context.app_frame_mut();
        let file_info_state = &mut app.file_info_state;

        let text = match self.info {
            FileInfoType::Info(info) => {
                let mapping = &info.source_mapping;

                let text: Text = if mapping.is_empty() {
                    vec!["File contains empty sourcemap (both \"sources\" and \"mappings\" arrays are empty).".into()]
                        .into()
                } else {
                    let sources_root = mapping.sources_root();

                    let source_file_len = mapping.actual_source_file_len();

                    let mut lines = vec![
                        Line::from(vec![
                            "File size: ".into(),
                            format_bytes(source_file_len).highlight(),
                            ".".into(),
                        ]),
                        Line::from(vec![
                            "Number of files: ".into(),
                            info.info_by_file.len().to_string().highlight(),
                            ".".into(),
                        ]),
                        Line::from(vec![
                            "Size contribution per file (all paths are relative to ".into(),
                            sources_root.bold(),
                            "):".into(),
                        ]),
                    ];

                    let mut info_by_file = info.info_by_file.iter().collect::<Vec<&SourceMappingFileInfo>>();
                    info_by_file.sort_by_key(|i| i.bytes);

                    for file_info in info_by_file.iter().rev() {
                        lines.push(
                            vec![
                                "- ".into(),
                                without_relative_part(info.get_file_name(file_info.file)).bold(),
                                ", size ".into(),
                                format_bytes(file_info.bytes as u64).highlight(),
                                " (".into(),
                                format_percentage(file_info.bytes as u64, source_file_len).highlight2(),
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
                            format_percentage(sum_bytes, source_file_len).highlight2(),
                            ")".into(),
                        ]
                        .into(),
                    );

                    let rest = source_file_len - sum_bytes;

                    lines.push(
                        vec![
                            "Remaining size taken by preamble, imports, whitespace, comments, etc.: ".into(),
                            format_bytes(rest).highlight(),
                            " (".into(),
                            format_percentage(rest, source_file_len).highlight2(),
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

        let block = get_block(is_focused);

        let block_inner = block.inner(rect);

        let height = calculate_height(&text, block.clone(), rect);

        file_info_state.max_height = block_inner.height;
        file_info_state.text_height = height;

        frame.render_widget(
            Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: true })
                .scroll((file_info_state.scroll, 0)),
            rect,
        );

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(file_info_state.max_scroll() as usize).position(file_info_state.scroll as usize);

        frame.render_stateful_widget(
            scrollbar,
            rect.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn get_block<'a>(is_focused: bool) -> Block<'a> {
    let mut block = default_block();
    if is_focused {
        block = block
            .border_style(Style::default().yellow())
            .title(Title::from(Line::from(keybindings!("t""ree toggle"))).position(Position::Bottom));
    }

    block
}

pub enum FileInfoViewType {
    Tree,
    Paragraph,
}

pub struct FileInfoState {
    pub view_type: FileInfoViewType,
    pub tree_state: TreeState,
    // paragraph state
    pub scroll: u16,
    pub text_height: u16,
    pub max_height: u16,
}

impl FileInfoState {
    fn max_scroll(&self) -> u16 {
        self.text_height.saturating_sub(self.max_height)
    }
}

impl Default for FileInfoState {
    fn default() -> Self {
        let tree_state = TreeState::default().initial_expansion_depth(2);

        Self {
            scroll: 0,
            text_height: 0,
            max_height: 0,
            tree_state,
            view_type: FileInfoViewType::Tree,
        }
    }
}

impl FocusableWidgetState for FileInfoState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match event.code {
            KeyCode::Char('t') => {
                self.view_type = match self.view_type {
                    FileInfoViewType::Tree => FileInfoViewType::Paragraph,
                    FileInfoViewType::Paragraph => FileInfoViewType::Tree,
                };
            }
            _ => match self.view_type {
                FileInfoViewType::Tree => self.handle_tree_events(event),
                FileInfoViewType::Paragraph => self.handle_paragraph_events(event),
            },
        }

        if matches!(event.code, KeyCode::Esc) {
            self.tree_state.list_state.select(None);
            HandleEventResult::ChangeFocus(FocusableWidget::FileList)
        } else {
            HandleEventResult::KeepFocus
        }
    }

    fn on_focus(&mut self) {
        if matches!(self.view_type, FileInfoViewType::Tree) {
            self.tree_state.list_state.select(Some(0));
        }
    }
}

impl FileInfoState {
    fn handle_tree_events(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.tree_state.next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.tree_state.previous();
            }
            KeyCode::Enter => {
                self.tree_state.toggle_selected();
            }
            _ => {}
        }
    }

    fn handle_paragraph_events(&mut self, event: KeyEvent) {
        let max_scroll = self.max_scroll();
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
    }
}

fn calculate_height(text: &Text, block: Block, area: Rect) -> u16 {
    // Total area of a paragraph must fit into u16, so height of the rect is computed
    // accordingly using max line width.
    let area = Rect::new(area.x, area.y, area.width, u16::MAX / area.width);
    let mut buffer = Buffer::empty(area);

    let paragraph = Paragraph::new(text.clone())
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((0, 0));

    paragraph.render(area, &mut buffer);

    for y in buffer.area.top()..buffer.area.bottom() {
        let x = buffer.area.left() + 1;

        if buffer.get(x, y).symbol() == " " {
            return y - 1 - area.y;
        }
    }

    0
}
