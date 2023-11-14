use ratatui::{
    prelude::{Alignment, Rect},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};

pub fn centered_text(f: &mut Frame, text: &str, rect: Rect) {
    f.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::new(0, 0, rect.height / 2, 0)),
            )
            .alignment(Alignment::Center),
        rect,
    );
}
