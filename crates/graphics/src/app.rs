use crate::{
    ContentManager, Error, WindowRoot, commands::CommandBuffer, rendering::Gpu, types::Bounds,
    widget::FrameContext,
};
use glam::Vec2;

pub struct App {
    pub(crate) frontends: Vec<Box<dyn WindowRoot>>,
    pub(crate) requested_frontends: Vec<Box<dyn WindowRoot>>,

    content: ContentManager,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self {
            frontends: vec![],
            requested_frontends: vec![],
            content: ContentManager::default(),
        }
    }

    pub(crate) fn dispatch_queue(&mut self, gpu: &Gpu) -> Result<(), Error> {
        self.content.dispatch_queue(gpu)
    }

    pub fn content_manager(&mut self) -> &mut ContentManager {
        &mut self.content
    }

    pub fn add_window(&mut self, mut window: Box<dyn WindowRoot>) {
        window.setup(self);
        self.requested_frontends.push(window);
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

    pub(crate) fn tick_render_frontend(&mut self, index: usize) -> CommandBuffer {
        let frontend = &mut self.frontends[index];
        let root = frontend.root_mut();
        let mut commands = CommandBuffer::new(&self.content);
        root.draw(&mut commands);
        commands.pack_active_group();
        commands
    }
}
