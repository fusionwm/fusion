use crate::ContentManager;
use bitflags::bitflags;
use glam::Vec2;
use wl_client::ButtonState;

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Anchor: u8 {
        const Left   = 1 << 0;
        const Right  = 1 << 1;
        const Top    = 1 << 2;
        const Bottom = 1 << 3;
        const Center = 1 << 4;

        const VerticalCenter   = 1 << 5;
        const HorizontalCenter = 1 << 6;
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum DesiredSize {
    Exact(Vec2),
    ExactY(f32),
    ExactX(f32),
    #[default]
    Fill,
    Ignore,
}

#[derive(Default)]
pub struct FrameContext {
    pub(crate) delta_time: f64,
    pub(crate) position: Vec2,
    pub(crate) buttons: ButtonState,
}

impl FrameContext {
    #[must_use]
    pub const fn delta_time(&self) -> f64 {
        self.delta_time
    }

    #[must_use]
    pub const fn position(&self) -> Vec2 {
        self.position
    }

    #[must_use]
    pub const fn buttons(&self) -> ButtonState {
        self.buttons
    }
}

pub trait Context: Send + Sync + Default + Sized + 'static {
    fn execute(&self, content: &mut ContentManager);
}

pub struct Sender<C: Context> {
    inner: Vec<C>,
}

impl<C: Context> Default for Sender<C> {
    fn default() -> Self {
        Self {
            inner: Vec::with_capacity(32),
        }
    }
}
