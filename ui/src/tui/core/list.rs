use std::cmp::Ordering;

use ratatui::widgets::ListState;

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

    pub fn selected_item(&self) -> Option<&T> {
        match self.state.selected() {
            Some(i) => self.items.get(i),
            None => None,
        }
    }

    pub fn sort(&mut self, mut compare: impl FnMut(&T, &T) -> Ordering, sort_order: SortOrder) {
        self.items.sort_by(|a, b| {
            let result = compare(a, b);

            match sort_order {
                SortOrder::Asc => result,
                SortOrder::Desc => result.reverse(),
            }
        });
    }
}

#[derive(Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn reverse(&self) -> Self {
        match self {
            SortOrder::Asc => SortOrder::Desc,
            SortOrder::Desc => SortOrder::Asc,
        }
    }
}
