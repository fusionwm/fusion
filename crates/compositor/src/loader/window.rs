use graphics::{
    WindowHandle,
    commands::{CommandBuffer, DrawRectCommand},
    graphics::Graphics,
    reexports::DesktopOptions,
    types::{Argb8888, Corners, Paint, PainterContext, Stroke},
    widget::{Anchor, Widget},
    window::WindowRequest,
};
use graphics_widgets::{row::Row, slider::Slider, text::Text};

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

fn slider_handle(ctx: &PainterContext, out: &mut CommandBuffer) {
    out.push(DrawRectCommand {
        rect: ctx.bounds,
        color: Argb8888::new(240, 240, 240, 255).into(),
        stroke: Stroke {
            color: [
                Argb8888::new(150, 150, 150, 255),
                Argb8888::new(150, 150, 150, 255),
                Argb8888::new(150, 150, 150, 255),
                Argb8888::new(150, 150, 150, 255),
            ],
            width: 1.0,
        },
        corners: Corners::NONE,
    });
}

pub fn test(graphics: &mut Graphics) {
    let mut label = Text::new();
    //label.anchor = Anchor::Center;
    label.set_text("Loading...");

    let mut slider = Slider::default();
    slider.style.handle.normal.background = Paint::Custom(Box::new(slider_handle));

    let mut row = Row::new();
    row.anchor = Anchor::Center;
    row.background = Argb8888::BLACK.into();
    row.content_mut().push(Box::new(label));
    row.content_mut().push(Box::new(slider));

    let mut label = Text::new();
    label.set_text("AGA");
    //row.content_mut().push(Box::new(label));

    let window = LoaderWindow::new(
        WindowRequest::new("Loader window").desktop(DesktopOptions::default()),
        Box::new(row),
    );

    //let window = LoaderWindow::new(
    //    WindowRequest::new("Loader window").background(SpecialOptions {
    //        anchor: graphics::reexports::Anchor::Bottom,
    //        exclusive_zone: 600,
    //        target: TargetMonitor::Primary,
    //    }),
    //    Box::new(row),
    //);

    graphics.add_window(Box::new(window));
}
