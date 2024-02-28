use ratatui::{
    prelude::{Alignment, Rect},
    style::Stylize,
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};

pub fn centered_text(f: &mut Frame, text: &str, rect: Rect) {
    f.render_widget(
        Paragraph::new(text)
            .block(default_block().padding(Padding::new(0, 0, rect.height / 2, 0)))
            .alignment(Alignment::Center),
        rect,
    );
}

pub fn default_block<'a>() -> Block<'a> {
    Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
}

pub fn default_scrollbar<'a>() -> Scrollbar<'a> {
    Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
}

pub trait CustomStyles<'a, T>: Stylize<'a, T> {
    fn highlight(self) -> T {
        self.cyan()
    }

    fn highlight2(self) -> T {
        self.green()
    }

    fn error(self) -> T {
        self.red()
    }

    fn key(self) -> T {
        self.yellow()
    }
}

impl<'a, A, T: Stylize<'a, A>> CustomStyles<'a, A> for T {}
