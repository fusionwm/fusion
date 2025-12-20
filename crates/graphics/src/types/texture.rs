use crate::{
    types::{Argb8888, Color},
    Handle,
};

#[derive(Debug, Clone)]
pub struct Texture {
    pub color: Color,
    pub handle: Handle,
}

impl Texture {
    #[must_use]
    pub const fn new(handle: Handle) -> Self {
        Self {
            color: Color::Simple(Argb8888::WHITE),
            handle,
        }
    }

    #[must_use]
    pub fn with_color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}
