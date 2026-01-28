use std::any::Any;

use crate::{
    ContentManager,
    commands::CommandBuffer,
    types::{Bounds, styling::StyleSheet},
};
use bitflags::bitflags;
use glam::Vec2;
use wl_client::ButtonState;

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Anchor: u8 {
        const Left   = 1 << 0;
        const Right  = 1 << 1;
        const Top    = 1 << 2;
        const Bottom = 1 << 3;
        const Center = 1 << 4;

        const VerticalCenter   = 1 << 5;
        const HorizontalCenter = 1 << 6;
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum DesiredSize {
    Exact(Vec2),
    ExactY(f32),
    ExactX(f32),
    #[default]
    Fill,
    Ignore,
}

#[derive(Default)]
pub struct FrameContext {
    pub(crate) delta_time: f64,
    pub(crate) position: Vec2,
    pub(crate) buttons: ButtonState,
}

impl FrameContext {
    #[must_use]
    pub const fn delta_time(&self) -> f64 {
        self.delta_time
    }

    #[must_use]
    pub const fn position(&self) -> Vec2 {
        self.position
    }

    #[must_use]
    pub const fn buttons(&self) -> ButtonState {
        self.buttons
    }
}

pub trait Context: Send + Sync + Default + Sized + 'static {
    fn execute(&self, content: &mut ContentManager);
}

impl Context for () {
    fn execute(&self, content: &mut ContentManager) {}
}

pub struct Sender<C: Context> {
    inner: Vec<C>,
}

impl<C: Context> Default for Sender<C> {
    fn default() -> Self {
        Self {
            inner: Vec::with_capacity(32),
        }
    }
}

#[allow(unused)]
pub trait Queryable {
    fn id(&self) -> Option<&str>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_element_dyn(&self, id: &str) -> Option<&dyn Any> {
        None
    }
    fn get_mut_element_dyn(&mut self, id: &str) -> Option<&mut dyn Any> {
        None
    }
}

impl Queryable for Vec<Box<dyn Widget>> {
    fn id(&self) -> Option<&str> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_element_dyn(&self, id: &str) -> Option<&dyn Any> {
        for element in self {
            let element = element.get_element_dyn(id);
            if element.is_some() {
                return element;
            }
        }

        None
    }

    fn get_mut_element_dyn(&mut self, id: &str) -> Option<&mut dyn Any> {
        for element in self.iter_mut() {
            let element = element.get_mut_element_dyn(id);
            if element.is_some() {
                return element;
            }
        }

        None
    }
}

#[derive(Default)]
pub struct Empty;

impl Widget for Empty {
    fn desired_size(&self) -> DesiredSize {
        DesiredSize::Ignore
    }

    fn anchor(&self) -> Anchor {
        Anchor::default()
    }

    fn draw<'frame, 'theme>(&'frame self, _: &'theme StyleSheet, _: &mut CommandBuffer<'frame>) {}
    fn layout(&mut self, _: Bounds) {}
    fn update(&mut self, _: &FrameContext) {}
}

impl Queryable for Empty {
    fn id(&self) -> Option<&str> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Widget for Option<Box<dyn Widget>> {
    fn desired_size(&self) -> DesiredSize {
        match self {
            Some(widget) => widget.desired_size(),
            None => DesiredSize::Ignore,
        }
    }

    fn anchor(&self) -> Anchor {
        match self {
            Some(widget) => widget.anchor(),
            None => Anchor::default(),
        }
    }

    fn draw<'frame>(&'frame self, stylesheet: &StyleSheet, out: &mut CommandBuffer<'frame>) {
        if let Some(widget) = self {
            widget.draw(stylesheet, out);
        }
    }

    fn layout(&mut self, bounds: Bounds) {
        if let Some(widget) = self {
            widget.layout(bounds);
        }
    }

    fn update(&mut self, ctx: &FrameContext) {
        if let Some(widget) = self {
            widget.update(ctx);
        }
    }
}

impl Queryable for Option<Box<dyn Widget>> {
    fn id(&self) -> Option<&str> {
        match self {
            Some(widget) => widget.id(),
            None => None,
        }
    }

    fn as_any(&self) -> &dyn Any {
        match self {
            Some(widget) => widget.as_any(),
            None => self,
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        match self {
            Some(widget) => widget.as_any_mut(),
            None => self,
        }
    }

    fn get_element_dyn(&self, id: &str) -> Option<&dyn Any> {
        match self {
            Some(widget) => widget.get_element_dyn(id),
            None => None,
        }
    }

    fn get_mut_element_dyn(&mut self, id: &str) -> Option<&mut dyn Any> {
        match self {
            Some(widget) => widget.get_mut_element_dyn(id),
            None => None,
        }
    }
}

pub trait Widget: Queryable + Send + Sync {
    fn desired_size(&self) -> DesiredSize;
    fn anchor(&self) -> Anchor;
    fn draw<'frame>(&'frame self, stylesheet: &StyleSheet, out: &mut CommandBuffer<'frame>);
    fn layout(&mut self, bounds: Bounds);
    fn update(&mut self, ctx: &FrameContext);
}

pub trait Container: Widget {
    fn add_child(&mut self, child: Box<dyn Widget>);
    fn children(&self) -> &[Box<dyn Widget>];
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>];
}
