use crate::{
    ContentManager, WindowHandle, commands::CommandBuffer, types::Bounds, widget::FrameContext,
};
use glam::Vec2;

#[derive(Default)]
pub struct Graphics {
    pub(crate) frontends: Vec<Box<dyn WindowHandle>>,
    pub(crate) requested_frontends: Vec<Box<dyn WindowHandle>>,
}

impl Graphics {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            frontends: vec![],
            requested_frontends: vec![],
        }
    }

    pub fn add_window(&mut self, mut window: Box<dyn WindowHandle>) {
        window.setup(self);
        self.requested_frontends.push(window);
    }

    // TODO: Возможно, стоит сделать метод более безопасным
    pub fn destroy_window(&mut self, window: *const dyn WindowHandle) {
        let index = self
            .requested_frontends
            .iter()
            .enumerate()
            .find(|&(_, b)| std::ptr::eq(b.as_ref(), window))
            .map(|(i, _)| i);

        if let Some(index) = index {
            self.requested_frontends.remove(index);
            return;
        }

        let index = self
            .frontends
            .iter()
            .enumerate()
            .find(|&(_, b)| std::ptr::eq(b.as_ref(), window))
            .map(|(i, _)| i);

        if let Some(index) = index {
            self.frontends.remove(index);
            return;
        }

        panic!("Window '{window:#?}' not found");
    }

    pub(crate) fn tick_logic_frontend(
        &mut self,
        index: usize,
        window_width: f32,
        window_height: f32,
        frame: &FrameContext,
    ) {
        let frontend = &mut self.frontends[index];
        let root = frontend.root_mut();
        root.update(frame);
        root.layout(Bounds::new(
            Vec2::ZERO,
            Vec2::new(window_width, window_height),
        ));
    }

    pub(crate) fn tick_render_frontend<'a>(
        &'a mut self,
        content: &'a ContentManager,
        index: usize,
    ) -> CommandBuffer<'a> {
        let frontend = &mut self.frontends[index];
        let root = frontend.root_mut();
        let mut commands = CommandBuffer::new(content);
        root.draw(&mut commands);
        commands.pack_active_group();
        commands
    }
}
