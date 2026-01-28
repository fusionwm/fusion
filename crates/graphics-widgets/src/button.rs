use graphics::{
    commands::{CommandBuffer, DrawRectCommand, DrawTextureCommand},
    glam::Vec2,
    types::{Argb8888, Bounds, Corners, Paint, PainterContext, Spacing, Stroke},
    widget::{Anchor, Context, DesiredSize, FrameContext, Widget},
};
use graphics_derive::Queryable;

#[derive(Default, Debug, Clone, Copy)]
enum ButtonFsm {
    #[default]
    Normal,
    Hovered,
    Pressed,
    PressedOutside,
}

#[derive(Clone)]
pub struct ButtonStyle {
    pub background: Paint,
    pub stroke: Stroke,
    pub corners: Corners,
}

impl ButtonStyle {
    pub(crate) fn normal() -> Self {
        Self {
            background: Argb8888::LIGHT_GRAY.into(),
            stroke: Stroke {
                color: [Argb8888::DARK_GRAY; 4],
                width: 1.0,
            },
            corners: Corners::DEFAULT,
        }
    }

    pub(crate) fn hover() -> Self {
        Self {
            background: Argb8888::new(230, 230, 230, 255).into(),
            stroke: Stroke {
                color: [Argb8888::BLUE; 4],
                width: 1.0,
            },
            corners: Corners::DEFAULT,
        }
    }

    pub(crate) fn pressed() -> Self {
        Self {
            background: Argb8888::GRAY.into(),
            stroke: Stroke {
                color: [Argb8888::DARK_GRAY; 4],
                width: 1.0,
            },
            corners: Corners::DEFAULT,
        }
    }
}

#[derive(Default)]
pub struct ButtonMock;
impl<C: Context> ButtonCallbacks<C> for ButtonMock {}

#[allow(dead_code, unused_variables)]
pub trait ButtonCallbacks<C: Context>: Default + Send + Sync + 'static {
    fn on_enter(&self) {}
    fn on_exit(&self) {}
    fn on_press(&self) {}
    fn on_clicked(&self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Alignment {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Queryable)]
pub struct Button<C = (), CB = ButtonMock>
where
    C: Context,
    CB: ButtonCallbacks<C>,
{
    id: Option<String>,
    pub size: Vec2,
    pub normal: ButtonStyle,
    pub hover: ButtonStyle,
    pub pressed: ButtonStyle,
    pub alignment: Alignment,
    pub padding: Spacing,
    pub anchor: Anchor,

    bounds: Bounds,
    state: ButtonFsm,

    callbacks: CB,
    #[content]
    content: Option<Box<dyn Widget>>,
    _phantom: std::marker::PhantomData<C>,
}

impl<C, CB> Default for Button<C, CB>
where
    C: Context,
    CB: ButtonCallbacks<C>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C, CB> Button<C, CB>
where
    C: Context,
    CB: ButtonCallbacks<C>,
{
    fn new_internal(id: Option<String>) -> Self {
        Self {
            id,
            size: Vec2::new(30.0, 30.0),
            normal: ButtonStyle::normal(),
            hover: ButtonStyle::hover(),
            pressed: ButtonStyle::pressed(),
            content: None,
            callbacks: CB::default(),
            bounds: Bounds::ZERO,
            state: ButtonFsm::Normal,
            alignment: Alignment::Center,
            anchor: Anchor::Left,
            padding: Spacing {
                left: 2.0,
                right: 2.0,
                top: 2.0,
                bottom: 2.0,
            },
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_with_id(id: impl Into<String>) -> Self {
        Self::new_internal(Some(id.into()))
    }

    #[must_use]
    pub fn new() -> Self {
        Self::new_internal(None)
    }

    pub fn content_mut(&mut self) -> Option<&mut dyn Widget> {
        match self.content {
            Some(ref mut boxed) => Some(&mut **boxed),
            None => None,
        }
    }

    pub fn content(&self) -> Option<&dyn Widget> {
        match self.content {
            Some(ref boxed) => Some(&**boxed),
            None => None,
        }
    }

    pub fn bounds(&self) -> Bounds {
        self.bounds
    }
}

impl<C, CB> Widget for Button<C, CB>
where
    C: Context,
    CB: ButtonCallbacks<C>,
{
    fn anchor(&self) -> Anchor {
        self.anchor
    }

    fn desired_size(&self) -> DesiredSize {
        DesiredSize::Exact(self.size)
    }

    fn draw<'frame>(&'frame self, out: &mut CommandBuffer<'frame>) {
        let style = match self.state {
            ButtonFsm::Normal => &self.normal,
            ButtonFsm::Hovered => &self.hover,
            ButtonFsm::Pressed | ButtonFsm::PressedOutside => &self.pressed,
        };

        match &style.background {
            Paint::Color(color) => {
                out.push(
                    DrawRectCommand::from_bounds(self.bounds)
                        .with_color(color.clone())
                        .with_stroke(style.stroke)
                        .with_corners(style.corners),
                );
            }
            Paint::Texture(texture) => out.push(
                DrawTextureCommand::from_bounds(texture.clone(), self.bounds)
                    .with_stroke(style.stroke)
                    .with_corners(style.corners),
            ),
            Paint::Custom(custom) => custom.draw(
                &PainterContext {
                    bounds: self.bounds,
                    border: style.stroke,
                    corners: style.corners,
                },
                out,
            ),
        }

        self.content.draw(out);
    }

    fn layout(&mut self, bounds: Bounds) {
        //println!("Bounds: {:#?}", bounds);
        self.bounds = bounds;

        let content_size = match self.content.desired_size() {
            DesiredSize::Exact(min) => Vec2::new(
                min.x
                    .min(self.size.x - self.padding.left - self.padding.right),
                min.y
                    .min(self.size.y - self.padding.top - self.padding.bottom),
            ),
            DesiredSize::ExactY(y) => Vec2::new(
                self.size.x - self.padding.left - self.padding.right,
                y.max(self.size.y - self.padding.top - self.padding.bottom),
            ),
            DesiredSize::ExactX(x) => Vec2::new(
                x.max(self.size.x - self.padding.right - self.padding.left),
                self.size.y - self.padding.top - self.padding.bottom,
            ),
            DesiredSize::Fill => Vec2::new(
                self.size.x - self.padding.left - self.padding.right,
                self.size.y - self.padding.top - self.padding.bottom,
            ),
            DesiredSize::Ignore => return,
        };

        let content_x = match self.alignment {
            Alignment::TopLeft | Alignment::CenterLeft | Alignment::BottomLeft => {
                self.bounds.position.x + self.padding.left
            }
            Alignment::TopCenter | Alignment::Center | Alignment::BottomCenter => {
                self.bounds.position.x + (self.size.x - content_size.x) / 2.0
            }
            Alignment::TopRight | Alignment::CenterRight | Alignment::BottomRight => {
                self.bounds.position.x + self.size.x - content_size.x - self.padding.right
            }
        };

        let content_y = match self.alignment {
            Alignment::TopLeft | Alignment::TopCenter | Alignment::TopRight => {
                self.bounds.position.y + self.padding.top
            }
            Alignment::CenterLeft | Alignment::Center | Alignment::CenterRight => {
                self.bounds.position.y + (self.size.y - content_size.y) / 2.0
            }
            Alignment::BottomLeft | Alignment::BottomCenter | Alignment::BottomRight => {
                self.bounds.position.y + self.size.y - content_size.y - self.padding.bottom
            }
        };

        let content_rect = Bounds {
            position: Vec2::new(content_x, content_y),
            size: Vec2::new(content_size.x, content_size.y),
        };

        self.content.layout(content_rect);
    }

    fn update(&mut self, ctx: &FrameContext) {
        let is_inside = self.bounds.contains(ctx.position());
        let is_pressed = ctx.buttons().left();
        match self.state {
            ButtonFsm::Normal => {
                if is_inside {
                    self.state = ButtonFsm::Hovered;
                    //self.callbacks.on_enter(sender);
                }
            }

            ButtonFsm::Hovered => {
                if !is_inside {
                    self.state = ButtonFsm::Normal;
                    //self.callbacks.on_exit(sender);
                } else if is_pressed {
                    self.state = ButtonFsm::Pressed;
                    //self.callbacks.on_press(sender);
                }
            }

            ButtonFsm::Pressed => {
                if !is_pressed {
                    self.state = ButtonFsm::Hovered;
                    //self.callbacks.on_clicked(sender);
                } else if !is_inside {
                    self.state = ButtonFsm::PressedOutside;
                }
            }

            ButtonFsm::PressedOutside => {
                if !is_pressed {
                    self.state = ButtonFsm::Normal;
                    //self.callbacks.on_clicked(sender);
                    //self.callbacks.on_exit(sender);
                }
            }
        }

        self.content.update(ctx);
        //self.content.update(ctx, sender);
    }
}
