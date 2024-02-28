use std::cmp::Ordering;

use ratatui::widgets::{ListState, TableState};

pub trait SelectableList {
    fn selected(&self) -> Option<usize>;

    fn select(&mut self, index: Option<usize>);
}

pub trait ListOperations {
    fn len(&self) -> usize;

    fn selected(&self) -> Option<usize>;

    fn select_inner(&mut self, index: Option<usize>);

    fn next(&mut self) {
        let i = match self.selected() {
            Some(i) => {
                if i >= self.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.select_inner(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.selected() {
            Some(i) => {
                if i == 0 {
                    self.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.select_inner(Some(i));
    }

    fn has_selection(&self) -> bool {
        self.selected().is_some()
    }

    fn select(&mut self, index: usize) {
        self.select_inner(Some(index))
    }

    fn unselect(&mut self) {
        self.select_inner(None);
    }
}

pub struct StatefulList<S, T>
where
    S: SelectableList,
{
    pub state: S,
    pub items: Vec<T>,
}

impl SelectableList for ListState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }

    fn select(&mut self, index: Option<usize>) {
        self.select(index)
    }
}

impl SelectableList for TableState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }

    fn select(&mut self, index: Option<usize>) {
        self.select(index)
    }
}

impl<S: Default + SelectableList, T> StatefulList<S, T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<S, T> {
        StatefulList {
            state: S::default(),
            items,
        }
    }
}

impl<S: SelectableList, T> ListOperations for StatefulList<S, T> {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    fn select_inner(&mut self, index: Option<usize>) {
        self.state.select(index);
    }
}

impl<S: SelectableList, T> StatefulList<S, T> {
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

pub struct DummyList<S>
where
    S: SelectableList,
{
    pub state: S,
    pub len: usize,
}

impl<S: SelectableList> ListOperations for DummyList<S> {
    fn len(&self) -> usize {
        self.len
    }

    fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    fn select_inner(&mut self, index: Option<usize>) {
        self.state.select(index);
    }
}
