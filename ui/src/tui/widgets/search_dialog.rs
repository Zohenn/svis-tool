use crate::{
    tui::{
        core::{FocusableWidgetState, HandleEventResult},
        App, FocusableWidget,
    },
    utils::without_relative_part,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph},
};

use super::{
    dialog::DialogContent,
    file_list::{AnalyzeState, FileInfoType},
    input::{InputWidget, InputWidgetState},
    mapping_info::FileInfoState,
};

#[derive(Default)]
pub struct SearchDialogState {
    pub path_input: InputWidgetState,
}

impl DialogContent for SearchDialogState {
    fn vertical_constraints(&self, _area: Rect) -> Constraint {
        Constraint::Length(6)
    }

    fn modify_block<'block>(&self, block: Block<'block>) -> Block<'block> {
        block.padding(Padding::symmetric(2, 1))
    }

    fn render_content(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Length(1)]).split(area);

        let label = Line::from(" Find source file ");

        let input = InputWidget::new(true).label(label);

        InputWidget::frame_render(f, input, chunks[0], &mut self.path_input);

        f.render_widget(
            Paragraph::new("Enter submits, Esc cancels").alignment(Alignment::Center),
            chunks[1],
        );
    }
}

impl FocusableWidgetState for SearchDialogState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult {
        match event.code {
            KeyCode::Enter => HandleEventResult::Callback(Box::new(Self::callback)),
            KeyCode::Esc => HandleEventResult::ChangeFocus(FocusableWidget::FileList),
            _ => return self.path_input.handle_events(event),
        }
    }

    fn callback(app: &mut App) -> HandleEventResult {
        let search_value = app.search_dialog.path_input.value().to_lowercase();

        match &mut app.file_list_state.analyze_state {
            Some(AnalyzeState::Done(done_state)) => {
                let mut found_file = None;

                for (pos, file_info) in done_state.file_infos.items.iter().enumerate() {
                    let FileInfoType::Info(info) = file_info else {
                        continue;
                    };

                    let file = info
                        .source_mapping
                        .sources
                        .iter()
                        .find(|source| source.to_lowercase().contains(&search_value));

                    let Some(file) = file else {
                        continue;
                    };

                    found_file = Some((pos, file));
                }

                match found_file {
                    Some((pos, file)) => {
                        let file = without_relative_part(file).to_owned();
                        done_state.file_infos.select(pos);
                        app.search_dialog.path_input.reset();

                        app.file_info_state = FileInfoState::default();
                        app.file_info_state.tree_state.ensure_leaf_is_visible(&file);
                        app.file_info_state.tree_state.initial_highlight(&file);

                        return HandleEventResult::ChangeFocus(FocusableWidget::FileInfo);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        HandleEventResult::ChangeFocus(FocusableWidget::FileList)
    }
}
