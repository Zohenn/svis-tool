use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
    Frame,
};

use core::analyzer::SourceMappingFileInfo;

use crate::utils::{format_bytes, format_percentage, without_relative_part};

use super::{
    core::{FocusableWidgetState, HandleEventResult},
    file_list::FileInfoType,
    widget_utils::{default_block, CustomStyles},
    FocusableWidget,
};

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
