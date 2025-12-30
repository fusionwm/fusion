use glam::{Mat4, Vec2};
use wgpu::RenderPass;

use crate::{
    commands::DrawDispatcher,
    impl_corner_radii,
    rendering::{Gpu, instance::InstanceData, material::Material},
    types::{Bounds, Corners, Stroke, Texture},
};

pub struct DrawTextureCommand {
    bounds: Bounds,
    texture: Texture,
    stroke: Stroke,
    corner_radii: Corners,
}

impl DrawTextureCommand {
    #[must_use]
    pub const fn new(
        bounds: Bounds,
        texture: Texture,
        stroke: Stroke,
        corner_radii: Corners,
    ) -> Self {
        Self {
            bounds,
            texture,
            stroke,
            corner_radii,
        }
    }

    #[must_use]
    pub fn from_texture(texture: Texture, bounds: Bounds) -> Self {
        Self {
            bounds,
            texture,
            stroke: Stroke::default(),
            corner_radii: Corners::default(),
        }
    }
}

impl_corner_radii!(DrawTextureCommand);

impl DrawDispatcher for DrawTextureCommand {
    fn texture_handle(&self) -> crate::Handle {
        self.texture.handle.clone()
    }

    fn prepare(&mut self, projection: Mat4, material: &mut Material) {
        const UV: [Vec2; 4] = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];
        material.push(InstanceData::new_uv_2(
            UV,
            self.bounds.position,
            self.bounds.size,
            &self.texture.color,
            Some(self.stroke.clone()),
            self.corner_radii,
            projection,
        ));
    }

    fn finish(&self, material: &mut Material, gpu: &Gpu, renderpass: &mut RenderPass) {
        material.draw_instances(gpu, renderpass);
    }
}
