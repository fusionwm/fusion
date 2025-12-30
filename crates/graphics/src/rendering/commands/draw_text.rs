use fontdue::layout::Layout;
use glam::{Mat4, Vec2};
use wgpu::RenderPass;

use crate::{
    ContentManager, FontHandle, TextureHandle,
    commands::DrawDispatcher,
    rendering::{Gpu, Renderer, instance::InstanceData, material::Material},
    types::{Color, Corners},
};

pub struct DrawTextCommand<'frame> {
    size: u32,
    color: Color,
    position: Vec2,
    pub(super) font: &'frame FontHandle,
    layout: &'frame Layout,
}

impl<'frame> DrawTextCommand<'frame> {
    pub fn new(
        size: u32,
        color: impl Into<Color>,
        position: Vec2,
        font: &'frame FontHandle,
        layout: &'frame Layout,
    ) -> Self {
        DrawTextCommand {
            size,
            color: color.into(),
            position,
            font,
            layout,
        }
    }
}

impl DrawDispatcher for DrawTextCommand<'_> {
    fn texture_handle(&self) -> crate::Handle {
        crate::Handle::Texture(TextureHandle::default())
    }

    fn prepare(&mut self, projection: Mat4, material: &mut Material, content: &mut ContentManager) {
        let atlas =
            content.get_mut_font_atlas(self.font.inner.name().unwrap().to_string(), self.size);

        self.layout.glyphs().iter().for_each(|glyph| {
            match glyph.parent {
                ' ' | '\t' | '\n' | '\r' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {
                    return;
                }
                c if c.is_control() => return,
                _ => {}
            }

            let data = atlas.get_or_add_glyph(glyph.parent, self.size, &self.font.inner);
            material.push(InstanceData::new_uv_4(
                data.uv,
                Vec2::new(
                    (self.position.x + glyph.x).round(),
                    (self.position.y + glyph.y).round(),
                ),
                Vec2::new(data.metrics.width as f32, data.metrics.height as f32),
                &self.color,
                None,
                Corners::NONE,
                projection,
            ));
        });
    }

    fn finish(&self, material: &mut Material, gpu: &Gpu, renderpass: &mut RenderPass) {
        let set = pipeline
            .fonts
            .entry(self.font.inner.name().unwrap().to_string())
            .or_default();
        let atlas = set.get_atlas(self.size);

        let material = atlas.get_or_add_material(gpu);
        //renderpass.set_bind_group(0, &material.bind_group, &[]);
        pipeline.material.draw_instances(gpu, renderpass);
    }
}
