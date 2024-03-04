use ratatui::{
    prelude::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};

use crate::theme::{ERROR, FOCUS, HIGHLIGHT, HIGHLIGHT2, TEXT};

pub fn centered_text(f: &mut Frame, text: &str, rect: Rect) {
    f.render_widget(
        Paragraph::new(text)
            .block(default_block().padding(Padding::new(0, 0, rect.height / 2, 0)))
            .alignment(Alignment::Center),
        rect,
    );
}

pub fn default_block<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(TEXT))
}

pub fn default_scrollbar<'a>() -> Scrollbar<'a> {
    Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
}

pub trait CustomStyles<'a, T>: Stylize<'a, T> {
    fn highlight(self) -> T {
        self.fg(HIGHLIGHT)
    }

    fn highlight2(self) -> T {
        self.fg(HIGHLIGHT2)
    }

    fn error(self) -> T {
        self.fg(ERROR)
    }

    fn key(self) -> T {
        self.fg(FOCUS)
    }
}

impl<'a, A, T: Stylize<'a, A>> CustomStyles<'a, A> for T {}
