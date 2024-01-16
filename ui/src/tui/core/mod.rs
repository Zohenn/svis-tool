pub mod list;
pub mod tree;

pub use list::*;

use super::{App, FocusableWidget};
use crossterm::event::KeyEvent;

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
