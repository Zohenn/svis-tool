use crossterm::event::KeyEvent;
use ratatui::widgets::ListState;

use super::{App, FocusableWidget};

pub enum HandleEventResult {
    Blur,
    KeepFocus,
    ChangeFocus(FocusableWidget),
    Callback(fn(&mut App) -> HandleEventResult), //Box<dyn FnMut(&mut App) -> HandleEventResult>),
}

pub trait FocusableWidgetState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult;

    fn callback(_app: &mut App) -> HandleEventResult
    where
        Self: Sized,
    {
        HandleEventResult::KeepFocus
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn has_selection(&self) -> bool {
        self.state.selected().is_some()
    }

    pub fn selected_item(&self) -> Option<&T> {
        match self.state.selected() {
            Some(i) => self.items.get(i),
            None => None,
        }
    }
}
