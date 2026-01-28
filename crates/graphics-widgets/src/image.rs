use graphics::{
    Handle,
    commands::{CommandBuffer, DrawRectCommand, DrawTextureCommand},
    glam::Vec2,
    types::{Argb8888, Bounds, Corners, Stroke, Texture},
    widget::{Anchor, DesiredSize, FrameContext, Widget},
};
use graphics_derive::Queryable;

#[derive(Queryable)]
pub struct Image {
    id: Option<String>,
    pub handle: Option<Handle>,
    pub anchor: Anchor,
    pub size: Vec2,
    rect: Bounds,
}

impl Default for Image {
    fn default() -> Self {
        Self::new()
    }
}

impl Image {
    const fn new_internal(id: Option<String>) -> Self {
        Self {
            id,
            size: Vec2::ZERO,
            rect: Bounds::ZERO,
            anchor: Anchor::Left,
            handle: None,
        }
    }

    fn new_with_id(id: String) -> Self {
        Self::new_internal(Some(id))
    }

    fn new() -> Self {
        Self::new_internal(None)
    }
}

impl Widget for Image {
    fn desired_size(&self) -> DesiredSize {
        DesiredSize::Exact(self.size)
    }

    fn anchor(&self) -> Anchor {
        self.anchor
    }

    fn draw<'frame>(&'frame self, out: &mut CommandBuffer<'frame>) {
        if let Some(handle) = &self.handle {
            out.push(DrawTextureCommand::new(
                self.rect.clone(),
                Texture {
                    color: Argb8888::WHITE.into(),
                    handle: handle.clone(),
                },
                Stroke::NONE,
                Corners::NONE,
            ));
        } else {
            out.push(DrawRectCommand::new(
                self.rect.clone(),
                Argb8888::WHITE,
                Stroke::NONE,
                Corners::NONE,
            ));
        }
    }

    fn layout(&mut self, bounds: Bounds) {
        self.rect = bounds;
    }

    fn update(&mut self, _: &FrameContext) {}
}
