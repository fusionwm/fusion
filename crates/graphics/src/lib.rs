#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

//#[cfg(feature = "derive")]
//pub use toolkit_derive::*;

//pub mod headless;

mod content;
//mod debug;
mod error;
mod rendering;
pub mod types;

pub use content::*;
pub use fontdue;
pub use glam;

pub mod app;
//pub mod event_loop;
pub mod widget;
pub mod window;

use crate::{
    app::App,
    rendering::{Gpu, Renderer, commands::CommandBuffer},
    types::Bounds,
    widget::{DesiredSize, FrameContext},
    window::{Window, WindowPointer, WindowRequest},
};
pub use error::*;
pub use rendering::commands;
use std::{ffi::c_void, ptr::NonNull, sync::Arc, time::Instant};
use wayland_client::{Connection, EventQueue, Proxy};
pub use wl_client::window::TargetMonitor;
pub use wl_client::{
    Anchor,
    window::{DesktopOptions, SpecialOptions},
};
use wl_client::{WlClient, window::WindowLayer};

pub struct EventLoop {
    app: App,
    windows: Vec<Window>,

    client: WlClient,
    event_queue: EventQueue<WlClient>,
    display_ptr: NonNull<c_void>,

    gpu: Gpu,
}

impl EventLoop {
    pub fn new(app: App) -> Result<Self, Error> {
        let conn = Connection::connect_to_env()?;

        let display = conn.display();
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, Arc::new(String::new()));

        let mut client = WlClient::default();

        event_queue.roundtrip(&mut client)?; //Register objects
        event_queue.roundtrip(&mut client)?; //Register outputs

        //Fix egl error: BadDisplay
        let (display_ptr, gpu) = {
            let display_ptr = NonNull::new(display.id().as_ptr().cast::<c_void>())
                .ok_or(Error::DisplayNullPointer)?;
            let dummy = client.create_window_backend(qh, "dummy", 1, 1, WindowLayer::default());
            event_queue.roundtrip(&mut client)?; //Init dummy

            let dummy_ptr = dummy
                .lock()
                .map_err(|e| Error::LockFailed(e.to_string()))?
                .as_ptr();
            let ptr = WindowPointer::new(display_ptr, dummy_ptr);
            let gpu = Gpu::new(ptr)?;

            drop(dummy);

            client.destroy_window_backend("dummy");
            event_queue.roundtrip(&mut client)?; //Destroy dummy

            (display_ptr, gpu)
        };

        Ok(Self {
            app,
            windows: vec![],

            client,
            event_queue,
            display_ptr,

            gpu,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.init_windows_backends()?;

        let mut previous = Instant::now();
        let mut frame = FrameContext::default();

        loop {
            let current = Instant::now();
            let delta = current - previous;
            previous = current;

            frame.delta_time = delta.as_secs_f64();
            frame.position = self.client.pointer().position();
            frame.buttons = self.client.pointer().buttons();

            self.app.dispatch_queue(&self.gpu)?;

            for (i, window) in self.windows.iter_mut().enumerate() {
                let mut backend = window
                    .backend
                    .lock()
                    .map_err(|e| Error::LockFailed(e.to_string()))?;
                if backend.can_resize() {
                    window.configuration.width = backend
                        .width
                        .try_into()
                        .map_err(|_| Error::NegativeWidth(backend.width))?;

                    window.configuration.height = backend
                        .height
                        .try_into()
                        .map_err(|_| Error::NegativeHeight(backend.height))?;

                    self.gpu
                        .confugure_surface(&window.surface, &window.configuration);
                    backend.set_resized();
                }

                self.app.tick_logic_frontend(
                    i,
                    window.configuration.width as f32,
                    window.configuration.height as f32,
                    &frame,
                );

                backend.frame();
                if !backend.can_draw() {
                    continue;
                    //return Ok(());
                }

                let mut commands = self.app.tick_render_frontend(i);
                window.renderer.render(
                    &self.gpu,
                    &window.surface,
                    &mut commands,
                    window.configuration.width as f32,
                    window.configuration.height as f32,
                )?;
                backend.commit();
            }

            self.event_queue.blocking_dispatch(&mut self.client)?;
        }
    }

    fn init_windows_backends(&mut self) -> Result<(), Error> {
        if self.app.requested_frontends.is_empty() {
            return Ok(());
        }

        let requests = std::mem::take(&mut self.app.requested_frontends);
        let qh = self.event_queue.handle();
        requests.into_iter().try_for_each(|frontend| {
            let request = frontend.request();
            let backend = self.client.create_window_backend(
                qh.clone(),
                request.id,
                request.width,
                request.height,
                request.layer,
            );

            let (width, height, surface_ptr) = {
                let guard = backend.lock().unwrap();

                let width: u32 = guard.width.try_into().expect("width must be >= 0");
                let height: u32 = guard.height.try_into().expect("height must be >= 0");
                (width, height, guard.as_ptr())
            };

            let window_ptr = WindowPointer::new(self.display_ptr, surface_ptr);
            let (surface, configuration) = self.gpu.create_surface(window_ptr, width, height)?;
            let renderer = Renderer::new(&self.gpu, None, &surface)?;
            let window = Window::new(backend, surface, configuration, renderer);

            self.windows.push(window);
            self.app.frontends.push(frontend);

            Ok::<(), Error>(())
        })?;

        Ok(())
    }
}

pub trait WindowRoot {
    fn request(&self) -> WindowRequest;
    fn setup(&mut self, app: &mut App);
    fn root_mut(&mut self) -> &mut dyn Windowx;
    fn root(&self) -> &dyn Windowx;
}

pub trait Windowx {
    fn desired_size(&self) -> DesiredSize;
    fn anchor(&self) -> Anchor;
    fn draw<'frame>(&'frame self, out: &mut CommandBuffer<'frame>);
    fn layout(&mut self, bounds: Bounds);
    fn update(&mut self, ctx: &FrameContext);
}

pub trait Container {
    fn add_child(&mut self, child: Box<dyn Windowx>);
    fn children(&self) -> &[Box<dyn Windowx>];
    fn children_mut(&mut self) -> &mut [Box<dyn Windowx>];
}
