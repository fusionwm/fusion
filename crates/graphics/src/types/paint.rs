use core::fmt::Debug;
use derive_more::From;

use crate::{
    commands::CommandBuffer,
    types::{Argb8888, Bounds, Color, Corners, Stroke, Texture},
};

#[derive(From)]
pub enum Paint {
    Color(Color),
    Texture(Texture),
    Custom(Box<dyn Painter>),
}

impl Debug for Paint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = String::from("Fn(&mut CommandBuffer)");

        match self {
            Self::Color(arg0) => f.debug_tuple("Color").field(arg0).finish(),
            Self::Texture(arg0) => f.debug_tuple("Texture").field(arg0).finish(),
            Self::Custom(_) => f.debug_tuple("Callback").field(&str).finish(),
        }
    }
}

impl Clone for Paint {
    fn clone(&self) -> Self {
        match self {
            Self::Color(arg0) => Self::Color(arg0.clone()),
            Self::Texture(arg0) => Self::Texture(arg0.clone()),
            Self::Custom(arg0) => Self::Custom(arg0.clone_dyn()),
        }
    }
}

impl From<Argb8888> for Paint {
    fn from(argb8888: Argb8888) -> Self {
        Self::Color(argb8888.into())
    }
}

pub struct PainterContext {
    pub bounds: Bounds,
    pub border: Stroke,
    pub corners: Corners,
}

pub trait Painter: Send + Sync {
    fn draw(&self, ctx: &PainterContext, out: &mut CommandBuffer);
    fn clone_dyn(&self) -> Box<dyn Painter>;
}

impl<F> Painter for F
where
    F: Fn(&PainterContext, &mut CommandBuffer) + Send + Sync + Clone + 'static,
{
    #[inline]
    fn draw(&self, ctx: &PainterContext, out: &mut CommandBuffer) {
        self(ctx, out);
    }

    fn clone_dyn(&self) -> Box<dyn Painter> {
        Box::new(self.clone())
    }
}
