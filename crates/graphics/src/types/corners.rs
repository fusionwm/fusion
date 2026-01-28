use glam::Vec4;

#[derive(Debug, Clone, Copy)]
pub struct Corners {
    pub top_left: f32,
    pub bottom_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
}

impl Default for Corners {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<Corners> for Vec4 {
    fn from(value: Corners) -> Self {
        Vec4::new(
            value.top_left,
            value.bottom_left,
            value.top_right,
            value.bottom_right,
        )
    }
}

impl Corners {
    pub const NONE: Corners = Self {
        top_left: 0.0,
        bottom_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
    };

    pub const DEFAULT: Corners = Self {
        top_left: 6.0,
        bottom_left: 6.0,
        top_right: 6.0,
        bottom_right: 6.0,
    };

    #[must_use]
    pub const fn all(value: f32) -> Self {
        Self {
            top_left: value,
            bottom_left: value,
            top_right: value,
            bottom_right: value,
        }
    }
}
