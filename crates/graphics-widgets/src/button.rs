use graphics::{
    commands::CommandBuffer,
    glam::Vec2,
    types::{Bounds, Spacing, styling::StyleSheet},
    widget::{Anchor, Context, DesiredSize, FrameContext, Widget},
};
use graphics_derive::Queryable;

use crate::draw;

pub struct ButtonStyle {
    background: String,
    border: String,
    corners: String,
}

impl ButtonStyle {
    fn new(widget: &str, state: &str) -> Self {
        Self {
            background: format!("{widget}:{state}:background"),
            border: format!("{widget}:{state}:border"),
            corners: format!("{widget}:{state}:corners"),
        }
    }
}

pub struct ButtonStyles {
    normal: ButtonStyle,
    hover: ButtonStyle,
    pressed: ButtonStyle,
}

impl ButtonStyles {
    #[must_use]
    pub fn new(widget: &str) -> Self {
        Self {
            normal: ButtonStyle::new(widget, "normal"),
            hover: ButtonStyle::new(widget, "hover"),
            pressed: ButtonStyle::new(widget, "pressed"),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
enum ButtonFsm {
    #[default]
    Normal,
    Hovered,
    Pressed,
    PressedOutside,
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
    pub alignment: Alignment,
    pub padding: Spacing,
    pub anchor: Anchor,
    pub styles: ButtonStyles,

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
    fn new_internal(id: Option<String>, widget_name: Option<&str>) -> Self {
        Self {
            id,
            size: Vec2::new(30.0, 30.0),
            styles: ButtonStyles::new(widget_name.unwrap_or("button")),
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
        Self::new_internal(Some(id.into()), Some("button"))
    }

    #[must_use]
    pub fn new() -> Self {
        Self::new_internal(None, Some("button"))
    }

    pub fn override_name(&mut self, name: &str) {
        self.styles = ButtonStyles::new(name);
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

    fn draw<'frame>(&'frame self, stylesheet: &StyleSheet, out: &mut CommandBuffer<'frame>) {
        let style = match self.state {
            ButtonFsm::Normal => &self.styles.normal,
            ButtonFsm::Hovered => &self.styles.hover,
            ButtonFsm::Pressed | ButtonFsm::PressedOutside => &self.styles.pressed,
        };

        out.push(draw(
            self.bounds,
            stylesheet.get_component(&style.background),
            stylesheet.get_stroke_component(&style.border),
            stylesheet.get_corners_component(&style.corners),
        ));

        self.content.draw(stylesheet, out);
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
