use crate::{
    types::{Argb8888, Color, LinearGradient, Texture},
    Handle, SvgHandle, TextureHandle,
};
use derive_more::From;

#[derive(Debug, Clone, From)]
pub enum BackgroundStyle {
    Color(Color),
    Texture(Texture),
}

impl BackgroundStyle {
    pub const WHITE: BackgroundStyle = Self::Color(Color::Simple(Argb8888::WHITE));
}

impl Default for BackgroundStyle {
    fn default() -> Self {
        Self::Color(Argb8888::WHITE.into())
    }
}

impl From<TextureHandle> for BackgroundStyle {
    fn from(value: TextureHandle) -> Self {
        BackgroundStyle::Texture(Texture {
            color: Argb8888::WHITE.into(),
            handle: Handle::Texture(value),
        })
    }
}

impl From<SvgHandle> for BackgroundStyle {
    fn from(value: SvgHandle) -> Self {
        BackgroundStyle::Texture(Texture {
            color: Argb8888::WHITE.into(),
            handle: Handle::Svg(value),
        })
    }
}

impl From<Argb8888> for BackgroundStyle {
    fn from(value: Argb8888) -> Self {
        Self::Color(value.into())
    }
}
impl From<LinearGradient> for BackgroundStyle {
    fn from(value: LinearGradient) -> Self {
        Self::Color(value.into())
    }
}
