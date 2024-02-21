use ratatui::{layout::Rect, Frame};

use crate::tui::{App, FocusableWidget};

pub struct RenderContext<'app, 'frame: 'app, D> {
    app: &'app mut App,
    frame: &'app mut Frame<'frame>,
    rendered_widget: FocusableWidget,
    data: D,
}

impl<'app, 'frame> RenderContext<'app, 'frame, ()> {
    pub fn new(
        app: &'app mut App,
        frame: &'app mut Frame<'frame>,
        rendered_widget: FocusableWidget,
    ) -> RenderContext<'app, 'frame, ()> {
        Self {
            app,
            frame,
            rendered_widget,
            data: (),
        }
    }

    pub fn with_data<D>(self, data: D) -> RenderContext<'app, 'frame, D> {
        RenderContext {
            app: self.app,
            frame: self.frame,
            rendered_widget: self.rendered_widget,
            data,
        }
    }
}

impl<'context, 'app: 'context, 'frame: 'app, D> RenderContext<'app, 'frame, D> {
    pub fn app_mut(&mut self) -> &mut App {
        self.app
    }

    pub fn frame_mut(&'context mut self) -> &'context mut Frame<'frame> {
        self.frame
    }

    pub fn app_frame_mut(&'context mut self) -> (&'context mut App, &'context mut Frame<'frame>) {
        (self.app, self.frame)
    }

    pub fn rendered_widget(&self) -> FocusableWidget {
        self.rendered_widget
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn is_focused(&self) -> bool {
        self.app.focused_widget == Some(self.rendered_widget)
    }
}

pub trait CustomWidget {
    type Data;

    fn render<'widget, 'app: 'widget>(self, context: RenderContext<'app, '_, Self::Data>, rect: Rect);
}
