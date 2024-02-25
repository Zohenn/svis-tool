use ratatui::{layout::Rect, Frame};

use crate::tui::{App, FocusableWidget};

pub struct RenderContext<'app, 'frame: 'app> {
    app: &'app mut App,
    frame: &'app mut Frame<'frame>,
    rendered_widget: Option<FocusableWidget>,
}

impl<'context, 'app: 'context, 'frame: 'app> RenderContext<'app, 'frame> {
    pub fn new(
        app: &'app mut App,
        frame: &'app mut Frame<'frame>,
        rendered_widget: Option<FocusableWidget>,
    ) -> RenderContext<'app, 'frame> {
        Self {
            app,
            frame,
            rendered_widget,
        }
    }

    pub fn app(&self) -> &App {
        self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        self.app
    }

    pub fn frame_mut(&'context mut self) -> &'context mut Frame<'frame> {
        self.frame
    }

    pub fn app_frame_mut(&'context mut self) -> (&'context mut App, &'context mut Frame<'frame>) {
        (self.app, self.frame)
    }

    #[allow(dead_code)]
    pub fn rendered_widget(&self) -> Option<FocusableWidget> {
        self.rendered_widget
    }

    pub fn is_focused(&self) -> bool {
        self.app.focused_widget == self.rendered_widget
    }
}

pub trait CustomWidget {
    fn bound_state(&self) -> Option<FocusableWidget>;

    fn render<'widget, 'app: 'widget>(&self, context: RenderContext<'app, '_>, rect: Rect);
}
