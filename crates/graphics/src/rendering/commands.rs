use crate::{
    ContentManager, FontHandle,
    rendering::{Gpu, Renderer, instance::InstanceData},
    types::{Argb8888, Bounds, Color, Corners, Stroke, Texture},
};
use enum_dispatch::enum_dispatch;
use fontdue::layout::Layout;
use glam::Vec2;
use std::slice::IterMut;
use wgpu::RenderPass;

#[enum_dispatch(DrawCommand)]
pub(crate) trait DrawDispatcher {
    fn prepare(
        &mut self,
        pipeline: &mut Renderer,
        gpu: &Gpu,
        content: &ContentManager,
        renderpass: &mut RenderPass,
    ) -> u32;

    fn finish(
        &self,
        pipeline: &mut Renderer,
        gpu: &Gpu,
        content: &ContentManager,
        renderpass: &mut RenderPass,
        count: u32,
    );
}

pub struct DrawRectCommand {
    pub rect: Bounds,
    pub color: Color,
    pub stroke: Stroke,
    pub corners: Corners,
}

impl DrawRectCommand {
    pub fn new(rect: Bounds, color: impl Into<Color>, stroke: Stroke, corners: Corners) -> Self {
        Self {
            rect,
            color: color.into(),
            stroke,
            corners,
        }
    }

    #[must_use]
    pub const fn from_bounds(bounds: Bounds) -> Self {
        Self {
            rect: bounds,
            color: Color::Simple(Argb8888::WHITE),
            stroke: Stroke::NONE,
            corners: Corners::NONE,
        }
    }

    #[must_use]
    pub fn with_color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    #[must_use]
    pub const fn with_corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    #[must_use]
    pub const fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }
}

impl DrawDispatcher for DrawRectCommand {
    fn prepare(
        &mut self,
        pipeline: &mut Renderer,
        _: &Gpu,
        _: &ContentManager,
        _: &mut RenderPass,
    ) -> u32 {
        const UV: [Vec2; 4] = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];

        pipeline.material.push(InstanceData::new_uv_2(
            UV,
            self.rect.position.round(),
            self.rect.size.round(),
            &self.color,
            Some(self.stroke),
            self.corners,
            pipeline.projection,
        ));

        1
    }

    fn finish(
        &self,
        pipeline: &mut Renderer,
        gpu: &Gpu,
        _: &ContentManager,
        renderpass: &mut RenderPass,
        count: u32,
    ) {
        pipeline.material.draw_instances(gpu, renderpass, count);
    }
}

pub struct DrawTextureCommand {
    rect: Bounds,
    texture: Texture,
    stroke: Stroke,
    corners: Corners,
}

impl DrawTextureCommand {
    #[must_use]
    pub fn new(rect: Bounds, texture: Texture, stroke: Stroke, corners: Corners) -> Self {
        Self {
            rect,
            texture,
            stroke,
            corners,
        }
    }

    #[must_use]
    pub const fn from_bounds(texture: Texture, bounds: Bounds) -> Self {
        Self {
            rect: bounds,
            texture,
            stroke: Stroke::NONE,
            corners: Corners::NONE,
        }
    }

    #[must_use]
    pub const fn with_texture(mut self, texture: Texture) -> Self {
        self.texture = texture;
        self
    }

    #[must_use]
    pub const fn with_corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    #[must_use]
    pub const fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }
}

impl DrawDispatcher for DrawTextureCommand {
    fn prepare(
        &mut self,
        pipeline: &mut Renderer,
        _: &Gpu,
        content: &ContentManager,
        _: &mut RenderPass,
    ) -> u32 {
        const UV: [Vec2; 4] = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];

        let mut material = content.get_material(&self.texture.handle).as_unsafe();
        material.push(InstanceData::new_uv_2(
            UV,
            self.rect.position,
            self.rect.size,
            &self.texture.color,
            Some(self.stroke),
            self.corners,
            pipeline.projection,
        ));

        1
    }

    fn finish(
        &self,
        _: &mut Renderer,
        gpu: &Gpu,
        content: &ContentManager,
        renderpass: &mut RenderPass,
        count: u32,
    ) {
        let mut material = content.get_material(&self.texture.handle).as_unsafe();
        material.draw_instances(gpu, renderpass, count);
    }
}

pub struct DrawTextCommand<'frame> {
    size: u32,
    color: Color,
    position: Vec2,
    font: &'frame FontHandle,
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
    fn prepare(
        &mut self,
        pipeline: &mut Renderer,
        gpu: &Gpu,
        _: &ContentManager,
        _: &mut RenderPass,
    ) -> u32 {
        let set = pipeline
            .fonts
            .entry(self.font.inner.name().unwrap().to_string())
            .or_default();
        let atlas = set.get_atlas(self.size);

        self.layout.glyphs().iter().for_each(|glyph| {
            match glyph.parent {
                ' ' | '\t' | '\n' | '\r' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {
                    return;
                }
                c if c.is_control() => return,
                _ => {}
            }

            let data = atlas.get_or_add_glyph(glyph.parent, self.size, &self.font.inner);
            let material = atlas.get_mut_or_add_material(gpu);
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
                pipeline.projection,
            ));
        });

        self.layout.glyphs().len() as u32
    }

    fn finish(
        &self,
        pipeline: &mut Renderer,
        gpu: &Gpu,
        _: &ContentManager,
        renderpass: &mut RenderPass,
        count: u32,
    ) {
        let set = pipeline
            .fonts
            .entry(self.font.inner.name().unwrap().to_string())
            .or_default();
        let atlas = set.get_atlas(self.size);
        let material = atlas.get_mut_or_add_material(gpu);
        material.draw_instances(gpu, renderpass, count);
    }
}

#[enum_dispatch]
pub enum DrawCommand<'frame> {
    Rect(DrawRectCommand),
    Texture(DrawTextureCommand),
    Text(DrawTextCommand<'frame>),
}

impl DrawCommand<'_> {
    fn is_same_type(&self, other: &DrawCommand) -> bool {
        use DrawCommand::{Rect, Text, Texture};

        match (self, other) {
            (Rect(_), Rect(_)) | (Texture(_), Texture(_)) => true,
            (Text(a), Text(b)) => a.font == b.font,
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct PackedGroup<'frame> {
    inner: Vec<DrawCommand<'frame>>,
}

impl PackedGroup<'_> {
    pub fn prepare_frame(
        &mut self,
        pipeline: &mut Renderer,
        content: &ContentManager,
        gpu: &Gpu,
        renderpass: &mut RenderPass,
    ) {
        let len = self.inner.len();
        let mut count = 0;

        for (i, command) in self.inner.iter_mut().enumerate() {
            count += command.prepare(pipeline, gpu, content, renderpass);

            if i == len - 1 {
                command.finish(pipeline, gpu, content, renderpass, count);
                count = 0;
            }
        }
    }
}

pub struct CommandBuffer<'frame> {
    content: &'frame ContentManager,
    packed: Vec<PackedGroup<'frame>>,
    active: Vec<DrawCommand<'frame>>,
}

impl<'frame> CommandBuffer<'frame> {
    #[must_use]
    pub const fn new(content: &'frame ContentManager) -> Self {
        Self {
            content,
            packed: vec![],
            active: vec![],
        }
    }

    pub fn push(&mut self, command: impl Into<DrawCommand<'frame>>) {
        let command = command.into();
        let last = self.active.last();
        if let Some(last) = last
            && !last.is_same_type(&command)
        {
            self.pack_active_group();
        }
        self.active.push(command);
    }

    pub(crate) fn pack_active_group(&mut self) {
        let group = std::mem::take(&mut self.active);
        self.packed.push(PackedGroup { inner: group });
    }

    pub(crate) fn iter_mut(&mut self) -> CommandBufferIter<'_, 'frame> {
        CommandBufferIter {
            content: self.content,
            iter: self.packed.iter_mut(),
        }
    }
}

impl<'a, 'frame> IntoIterator for &'a mut CommandBuffer<'frame> {
    type Item = (&'frame ContentManager, &'a mut PackedGroup<'frame>);
    type IntoIter = CommandBufferIter<'a, 'frame>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct CommandBufferIter<'a, 'frame> {
    content: &'frame ContentManager,
    iter: IterMut<'a, PackedGroup<'frame>>,
}

impl<'a, 'frame> Iterator for CommandBufferIter<'a, 'frame> {
    type Item = (&'frame ContentManager, &'a mut PackedGroup<'frame>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|packed| (self.content, packed))
    }
}
