use super::Argb8888;

#[derive(Debug, Copy, Clone)]
pub struct Stroke {
    ///Left, Right, Top, Bottom
    pub color: [Argb8888; 4],
    pub width: f32,
}

impl Default for Stroke {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Stroke {
    pub const NONE: Stroke = Self {
        color: [Argb8888::TRANSPARENT; 4],
        width: 0.0,
    };

    pub const DEFAULT: Stroke = Self {
        color: [Argb8888::GRAY; 4],
        width: 1.0,
    };
}
