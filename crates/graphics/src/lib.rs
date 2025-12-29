#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

//pub mod headless;

mod content;
//mod debug;
mod error;
mod rendering;
pub mod types;

pub use content::*;
pub use fontdue;
pub use glam;

pub mod graphics;
pub mod widget;
pub mod window;

pub mod reexports {
    pub use wl_client::Anchor;
    pub use wl_client::window::{DesktopOptions, SpecialOptions, TargetMonitor, WindowLayer};
}

use crate::{
    graphics::Graphics,
    rendering::{Gpu, Renderer},
    widget::{FrameContext, Widget},
    window::{Window, WindowPointer, WindowRequest},
};
pub use error::*;
pub use rendering::commands;
use std::{
    ffi::c_void,
    os::unix::net::UnixStream,
    ptr::NonNull,
    sync::{Arc, Mutex},
};
use wayland_client::{Connection, DispatchError, EventQueue, Proxy, backend::WaylandError};
use wl_client::WlClient;
use wl_client::window::WindowLayer;

pub struct InternalClient {
    app: Arc<Mutex<Graphics>>,
    windows: Vec<Window>,

    client: WlClient,
    event_queue: EventQueue<WlClient>,
    display_ptr: NonNull<c_void>,

    gpu: Gpu,
}

impl InternalClient {
    pub fn new(graphics: Arc<Mutex<Graphics>>, stream: UnixStream) -> Result<Self, Error> {
        let conn = Connection::from_socket(stream)?;

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
            app: graphics,
            windows: vec![],

            client,
            event_queue,
            display_ptr,

            gpu,
        })
    }

    pub fn tick(&mut self, delta: f64) -> Result<(), Error> {
        self.init_windows_backends()?;
        //TODO Frame data
        let frame = FrameContext {
            delta_time: delta,
            position: self.client.pointer().position(),
            buttons: self.client.pointer().buttons(),
        };

        let mut app = self.app.lock().unwrap();
        app.dispatch_queue(&self.gpu)?;

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

            app.tick_logic_frontend(
                i,
                window.configuration.width as f32,
                window.configuration.height as f32,
                &frame,
            );

            backend.frame();
            if !backend.can_draw() {
                continue;
            }

            let mut commands = app.tick_render_frontend(i);
            window.renderer.render(
                &self.gpu,
                &window.surface,
                &mut commands,
                window.configuration.width as f32,
                window.configuration.height as f32,
            )?;
            backend.commit();
        }

        if let Err(err) = self.event_queue.blocking_dispatch(&mut self.client)
            && !matches!(&err, DispatchError::Backend(WaylandError::Io(_)))
        {
            return Err(err.into());
        }

        Ok(())
    }

    fn init_windows_backends(&mut self) -> Result<(), Error> {
        let mut app = self.app.lock().unwrap();
        if app.requested_frontends.is_empty() {
            return Ok(());
        }

        let requests = std::mem::take(&mut app.requested_frontends);
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
                let mut guard = backend.lock().unwrap();
                guard.commit();

                let width: u32 = guard.width.try_into().expect("width must be >= 0");
                let height: u32 = guard.height.try_into().expect("height must be >= 0");
                (width, height, guard.as_ptr())
            };

            let window_ptr = WindowPointer::new(self.display_ptr, surface_ptr);
            let (surface, configuration) = self.gpu.create_surface(window_ptr, width, height)?;

            let renderer = Renderer::new(&self.gpu, None, &surface)?;

            let window = Window::new(backend, surface, configuration, renderer);

            app.frontends.push(frontend);
            self.windows.push(window);

            Ok::<(), Error>(())
        })?;

        if let Err(err) = self.event_queue.blocking_dispatch(&mut self.client)
            && !matches!(&err, DispatchError::Backend(WaylandError::Io(_)))
        {
            return Err(err.into());
        }
        Ok(())
    }
}

pub trait WindowHandle: Send + Sync {
    //TODO remove this method
    fn request(&self) -> WindowRequest;
    fn setup(&mut self, app: &mut Graphics);
    fn root_mut(&mut self) -> &mut dyn Widget;
    fn root(&self) -> &dyn Widget;
}

//pub trait Context: Send + Sync + Default + Sized + 'static {
//    //fn execute(&self, content: &mut ContentManager, tree: &mut Tree<Self>);
//}
