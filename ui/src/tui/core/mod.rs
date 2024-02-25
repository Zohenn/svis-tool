pub mod custom_widget;
pub mod list;
pub mod tree;

pub use list::*;

use super::{App, FocusableWidget};
use crossterm::event::KeyEvent;

pub enum HandleEventResult {
    Blur,
    KeepFocus,
    ChangeFocus(FocusableWidget),
    Callback(Box<dyn Fn(&mut App) -> HandleEventResult>),
}

pub trait FocusableWidgetState {
    fn handle_events(&mut self, event: KeyEvent) -> HandleEventResult;

    fn on_focus(&mut self) {}

    fn callback(_app: &mut App) -> HandleEventResult
    where
        Self: Sized,
    {
        HandleEventResult::KeepFocus
    }
}

#[macro_export]
macro_rules! keybindings {
    ( $($key:literal$rest:literal $(; $($sep:expr),+ ;)?),+ ) => {
        vec![
            " ".into(),
            $(
                $key.key().into(),
                $rest.fg($crate::theme::TEXT).into(),
                $(
                    $(
                        $sep,
                    )+
                )?
            )+
            " ".into(),
        ]
    };
}
