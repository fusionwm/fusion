use crate::{
    Handle,
    types::{Argb8888, Color},
};

#[derive(Clone)]
pub struct Texture {
    pub color: Color,
    pub handle: Handle<crate::content::Texture>,
}

impl Texture {
    #[must_use]
    pub const fn new(handle: Handle<crate::content::Texture>) -> Self {
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
