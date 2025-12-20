use wayland_client::{protocol::wl_output::Transform as WTransform, WEnum};

#[derive(Debug)]
pub enum Transform {
    Normal0,
    Normal90,
    Normal180,
    Normal270,
    Flipped0,
    Flipped90,
    Flipped180,
    Flipped270,
    Custom(u32),
}

impl From<WEnum<WTransform>> for Transform {
    fn from(value: WEnum<WTransform>) -> Self {
        match value {
            WEnum::Value(t) => match t {
                WTransform::_90 => Transform::Normal90,
                WTransform::_180 => Transform::Normal180,
                WTransform::_270 => Transform::Normal270,
                WTransform::Flipped => Transform::Flipped0,
                WTransform::Flipped90 => Transform::Flipped90,
                WTransform::Flipped180 => Transform::Flipped180,
                WTransform::Flipped270 => Transform::Flipped270,
                WTransform::Normal | _ => Transform::Normal0,
            },
            WEnum::Unknown(v) => Transform::Custom(v),
        }
    }
}
