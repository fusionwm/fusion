use glam::Vec4;

#[derive(Clone)]
pub struct Corners {
    pub left_top: f32,
    pub left_bottom: f32,
    pub right_top: f32,
    pub right_bottom: f32,
}

impl Default for Corners {
    fn default() -> Self {
        Self {
            left_top: 2.0,
            left_bottom: 2.0,
            right_top: 2.0,
            right_bottom: 2.0,
        }
    }
}

impl From<Corners> for Vec4 {
    fn from(value: Corners) -> Self {
        Vec4::new(value.left_top, value.left_bottom, value.right_top, value.right_bottom)
    }
}

impl Corners {
    pub const NONE: Corners = Self {
        left_top: 0.0,
        left_bottom: 0.0,
        right_top: 0.0,
        right_bottom: 0.0,
    };
}
