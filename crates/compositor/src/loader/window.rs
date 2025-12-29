use graphics::{
    WindowHandle,
    graphics::Graphics,
    reexports::{SpecialOptions, TargetMonitor},
    types::Argb8888,
    widget::{Anchor, Widget},
    window::WindowRequest,
};
use graphics_widgets::{row::Row, text::Text};

pub struct LoaderWindow {
    request: WindowRequest,
    root: Box<dyn Widget>,
}

impl LoaderWindow {
    pub const fn new(request: WindowRequest, root: Box<dyn Widget>) -> Self {
        Self { request, root }
    }
}

impl WindowHandle for LoaderWindow {
    fn request(&self) -> WindowRequest {
        self.request.clone()
    }

    fn setup(&mut self, _app: &mut Graphics) {}

    fn root_mut(&mut self) -> &mut dyn Widget {
        self.root.as_mut()
    }

    fn root(&self) -> &dyn Widget {
        self.root.as_ref()
    }
}

pub fn test(graphics: &mut Graphics) {
    let mut label = Text::new();
    label.anchor = Anchor::Center;
    label.set_text("Loading...");

    let mut row = Row::new();
    row.background = Argb8888::BLACK.into();
    row.content_mut().push(Box::new(label));

    let window = LoaderWindow::new(
        WindowRequest::new("Loader window").background(SpecialOptions {
            anchor: graphics::reexports::Anchor::Bottom,
            exclusive_zone: 600,
            target: TargetMonitor::Primary,
        }),
        Box::new(row),
    );

    graphics.add_window(Box::new(window));
}
