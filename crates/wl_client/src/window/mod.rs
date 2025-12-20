mod pool;
mod transform;

use crate::WlClient;
pub(crate) use pool::ShmPool;
use smithay_client_toolkit::reexports::protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{Anchor, ZwlrLayerSurfaceV1},
};
use std::{ffi::c_void, ptr::NonNull, sync::Arc};
use transform::Transform;
use wayland_client::{
    Proxy, QueueHandle,
    protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase,
};

pub type WindowId = Arc<String>;

#[derive(Default, Debug, Clone)]
pub struct DesktopOptions {
    pub title: String,
    pub resizable: bool,
    pub decorations: bool,
}

#[derive(Debug, Clone)]
pub struct SpecialOptions {
    pub anchor: Anchor,
    pub exclusive_zone: u32,
    pub target: TargetMonitor,
}

#[derive(Debug, Clone)]
pub enum WindowLayer {
    Desktop(DesktopOptions),
    Top(SpecialOptions),
    Bottom(SpecialOptions),
    Overlay(SpecialOptions),
    Background(SpecialOptions),
}

impl Default for WindowLayer {
    fn default() -> Self {
        Self::Desktop(DesktopOptions::default())
    }
}

#[derive(Default, Debug, Clone)]
pub enum TargetMonitor {
    #[default]
    Primary,
    Name(String),
    Index(usize),
    All,
}

#[derive(Debug, Default)]
struct Unused {
    layer_surface: Option<ZwlrLayerSurfaceV1>,
    xdg_surface: Option<XdgSurface>,
    xdg_toplevel: Option<XdgToplevel>,
}

#[derive(Debug)]
pub struct Window {
    surface: WlSurface,
    buffer: WlBuffer,
    pool: ShmPool,
    qh: QueueHandle<WlClient>,
    pub id: Arc<String>,

    pub layer: WindowLayer,

    //Window size
    pub width: i32,
    pub height: i32,

    //Window transformation
    pub scale: i32,
    pub transform: Transform,

    pub(crate) can_draw: bool,
    pub(crate) can_resize: bool,

    unused: Unused,
}

impl Window {
    pub fn resize_buffer_if_needed(&mut self) {
        self.buffer = self
            .pool
            .create_buffer(0, self.width, self.height, &self.qh, &self.id);
    }

    pub fn destroy(self) {
        self.buffer.destroy();
        self.pool.destroy();
        self.surface.destroy();
        if let Some(surface) = self.unused.layer_surface {
            surface.destroy();
        }

        if let Some(surface) = self.unused.xdg_surface {
            surface.destroy();
        }

        if let Some(xdg_toplevel) = self.unused.xdg_toplevel {
            xdg_toplevel.destroy();
        }
    }

    #[must_use]
    pub fn new(
        ls: Option<&ZwlrLayerShellV1>, // 'Some' when WindowLayer is not a WindowLayer::Desktop
        xdg_wm_base: Option<&XdgWmBase>, // 'Some' when WindowLayer is a WindowLayer::Desktop

        qh: QueueHandle<WlClient>,
        id: WindowId,

        surface: WlSurface,
        pool: ShmPool,
        buffer: WlBuffer,

        width: i32,
        height: i32,
        layer: WindowLayer,
    ) -> Self {
        let mut instance = Self {
            surface,
            buffer,
            pool,
            qh,
            id,
            layer,
            width,
            height,
            scale: 1,
            transform: Transform::Normal0,
            can_draw: false,
            can_resize: false,
            unused: Unused::default(),
        };

        instance.init(ls, xdg_wm_base);
        instance.draw();
        instance.frame();
        instance
    }

    fn init(&mut self, ls: Option<&ZwlrLayerShellV1>, xdg_wm_base: Option<&XdgWmBase>) {
        match self.layer.clone() {
            WindowLayer::Desktop(_) => self.init_desktop(xdg_wm_base.unwrap()),
            WindowLayer::Top(options) => self.init_layer_shell(ls.unwrap(), Layer::Top, &options),
            WindowLayer::Bottom(options) => {
                self.init_layer_shell(ls.unwrap(), Layer::Bottom, &options);
            }
            WindowLayer::Overlay(options) => {
                self.init_layer_shell(ls.unwrap(), Layer::Overlay, &options);
            }
            WindowLayer::Background(options) => {
                self.init_layer_shell(ls.unwrap(), Layer::Background, &options);
            }
        }
    }

    fn init_layer_shell(&mut self, ls: &ZwlrLayerShellV1, layer: Layer, options: &SpecialOptions) {
        let layer_surface = ls.get_layer_surface(
            &self.surface,
            None, //TODO fix
            layer,
            self.id.as_ref().into(),
            &self.qh,
            self.id.clone(),
        );

        layer_surface.set_size(self.width as u32, self.height as u32);
        layer_surface.set_anchor(options.anchor);
        layer_surface.set_exclusive_zone(options.exclusive_zone as i32);

        self.unused.layer_surface = Some(layer_surface);
    }

    fn init_desktop(&mut self, xdg_wm_base: &XdgWmBase) {
        let xdg_surface = xdg_wm_base.get_xdg_surface(&self.surface, &self.qh, self.id.clone());
        let xdg_toplevel = xdg_surface.get_toplevel(&self.qh, self.id.clone());
        self.unused.xdg_surface = Some(xdg_surface);
        self.unused.xdg_toplevel = Some(xdg_toplevel);
    }

    #[must_use]
    pub fn can_draw(&self) -> bool {
        self.can_draw
    }

    pub fn frame(&self) {
        self.surface.frame(&self.qh, self.id.clone());
    }

    pub fn damage_buffer(&mut self) {
        self.surface.damage_buffer(0, 0, self.width, self.height);
    }

    pub fn commit(&mut self) {
        self.surface.damage_buffer(0, 0, self.width, self.height);
        self.surface.commit();
        self.can_draw = false;
    }

    pub fn draw(&mut self) {
        self.surface.attach(Some(&self.buffer), 0, 0);
        self.commit();
    }

    pub fn resize_pool_if_needed(&mut self) {
        let size = (self.width as u64 * 4) * self.height as u64;
        if self.pool.need_resize(size) {
            self.pool.resize(size);
        }
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, width: usize, pixel: (u8, u8, u8, u8)) {
        self.pool.write_pixel(x, y, width, pixel);
    }

    pub fn draw_text_at(&mut self, x: usize, y: usize, coverage: f32) {
        self.pool
            .draw_text_at(x, y, self.width as usize, self.height as usize, coverage);
    }

    #[must_use]
    pub fn as_ptr(&self) -> NonNull<c_void> {
        NonNull::new(self.surface.id().as_ptr().cast::<c_void>()).unwrap()
    }

    #[must_use]
    pub fn can_resize(&self) -> bool {
        self.can_resize
    }

    pub fn set_resized(&mut self) {
        self.can_resize = false;
    }
}
