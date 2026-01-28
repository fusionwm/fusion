use crate::types::{self, Argb8888, Corners, Stroke};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    uv: Vec4,
    size: Vec2,
    _padding0: [u32; 2],

    model: Mat4,

    color: Vec4,

    stroke_color_left: Vec4,
    stroke_color_right: Vec4,
    stroke_color_top: Vec4,
    stroke_color_bottom: Vec4,

    color_end: Vec4,

    corners: Vec4,

    //degree: f32,
    //use_gradient: u32,
    //support_stroke: u32,
    //stroke_width: f32,
    misc: Vec4,
}

impl InstanceData {
    pub fn new_uv_4(
        uv: Vec4,
        position: Vec2,
        size: Vec2,
        color: &types::Color,
        stroke: Option<Stroke>,
        corners: Corners,
        proj: Mat4,
    ) -> Self {
        let model = proj
            * Mat4::from_scale_rotation_translation(
                Vec3::new(size.x, size.y, 0.0),
                Quat::IDENTITY,
                Vec3::new(position.x, position.y, 0.0),
            );

        let (color, color_end, degree, use_gradient): (Vec4, Vec4, f32, u32) = match color {
            types::Color::Simple(argb8888) => {
                (argb8888.into(), Argb8888::TRANSPARENT.into(), 0.0, 0)
            }
            types::Color::LinearGradient(linear_gradient) => (
                (&linear_gradient.from).into(),
                (&linear_gradient.to).into(),
                linear_gradient.degree,
                1,
            ),
        };

        let (stroke_color, stroke_width, support_stroke) = {
            if let Some(stroke) = stroke {
                (
                    [
                        stroke.color[0].into(),
                        stroke.color[1].into(),
                        stroke.color[2].into(),
                        stroke.color[3].into(),
                    ],
                    stroke.width,
                    1,
                )
            } else {
                Default::default()
            }
        };

        //degree: f32,
        //use_gradient: u32,
        //support_stroke: u32,
        //stroke_width: f32,
        let misc = Vec4::new(
            degree,
            use_gradient as f32,
            support_stroke as f32,
            stroke_width,
        );

        Self {
            uv,
            size,

            model,
            color,

            stroke_color_left: stroke_color[0],
            stroke_color_right: stroke_color[1],
            stroke_color_top: stroke_color[2],
            stroke_color_bottom: stroke_color[3],
            color_end,
            misc,

            _padding0: Default::default(),
            corners: corners.into(),
        }
    }

    pub fn new_uv_2(
        uv: [Vec2; 4],
        position: Vec2,
        size: Vec2,
        color: &types::Color,
        stroke: Option<Stroke>,
        corners: Corners,
        proj: Mat4,
    ) -> Self {
        let u_min = uv[0].x.min(uv[1].x).min(uv[2].x).min(uv[3].x);
        let v_min = uv[0].y.min(uv[1].y).min(uv[2].y).min(uv[3].y);
        let u_max = uv[0].x.max(uv[1].x).max(uv[2].x).max(uv[3].x);
        let v_max = uv[0].y.max(uv[1].y).max(uv[2].y).max(uv[3].y);

        let uv_rect = Vec4::new(u_min, v_min, u_max, v_max);

        Self::new_uv_4(uv_rect, position, size, color, stroke, corners, proj)
    }
}
