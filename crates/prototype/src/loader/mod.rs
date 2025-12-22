mod state;

use std::{
    os::unix::net::UnixStream,
    sync::{Arc, Mutex, mpsc::Sender},
    time::Duration,
};

use calloop::{
    EventLoop, Interest, PostAction,
    generic::Generic,
    timer::{TimeoutAction, Timer},
};
use graphics::{InternalClient, graphics::Graphics};
use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker, element::surface::WaylandSurfaceRenderElement,
            gles::GlesRenderer,
        },
        winit::WinitEvent,
    },
    desktop::space::render_output,
    output::Mode,
    utils::{Rectangle, Transform},
};
use wayland_server::{Display, DisplayHandle};

pub use crate::loader::state::ClientState;
use crate::{
    compositor::{backend::Backend, window::WinitBackend},
    loader::state::LoaderState,
};

mod xd {
    use graphics::{
        Widget, WindowHandle,
        commands::{DrawCommand, DrawRectCommand},
        glam::Vec2,
        graphics::Graphics,
        reexports::{Anchor, SpecialOptions, TargetMonitor},
        types::{Argb8888, Bounds, Stroke},
        window::WindowRequest,
    };

    pub struct ModuleWindow {
        bounds: Bounds,
    }

    impl Default for ModuleWindow {
        fn default() -> Self {
            Self {
                bounds: Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
            }
        }
    }

    impl Widget for ModuleWindow {
        fn desired_size(&self) -> graphics::widget::DesiredSize {
            graphics::widget::DesiredSize::Fill
        }

        fn anchor(&self) -> graphics::reexports::Anchor {
            graphics::reexports::Anchor::Left
        }

        fn draw<'frame>(&'frame self, out: &mut graphics::commands::CommandBuffer<'frame>) {
            out.push(DrawCommand::Rect(DrawRectCommand::new(
                self.bounds.clone(),
                Argb8888::BLACK,
                Stroke::NONE,
            )));
        }

        fn layout(&mut self, bounds: graphics::types::Bounds) {
            self.bounds = bounds;
        }

        fn update(&mut self, _ctx: &graphics::widget::FrameContext) {}
    }

    pub struct DynamicWindow {
        request: WindowRequest,
        root: ModuleWindow,
    }

    impl WindowHandle for DynamicWindow {
        fn request(&self) -> graphics::window::WindowRequest {
            self.request.clone()
        }

        fn setup(&mut self, _app: &mut graphics::graphics::Graphics) {}

        fn root_mut(&mut self) -> &mut dyn graphics::Widget {
            &mut self.root
        }

        fn root(&self) -> &dyn graphics::Widget {
            &self.root
        }
    }

    pub fn test(graphics: &mut Graphics) {
        let window = Box::new(DynamicWindow {
            request: WindowRequest::new("Test Window")
                //.with_size(800, 600)
                .background(SpecialOptions {
                    anchor: Anchor::Bottom,
                    exclusive_zone: 600,
                    target: TargetMonitor::Primary,
                }),
            root: ModuleWindow::default(),
        });
        graphics.add_window(window);
    }
}

pub struct ClientSignal {
    inner: Sender<()>,
}

impl ClientSignal {
    pub fn stop(&self) {
        self.inner.send(()).unwrap();
    }
}

fn spawn_client_thread(
    graphics: Arc<Mutex<Graphics>>,
    client_stream: UnixStream,
) -> Result<ClientSignal, Box<dyn std::error::Error>> {
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    std::thread::Builder::new()
        .name("Client".into())
        .spawn(move || {
            let mut client = InternalClient::new(graphics, client_stream).unwrap();
            while receiver.try_recv().is_err() {
                if let Err(err) = client.tick(0.0) {
                    log::error!("[Client] {err}");
                }
            }
        })?;

    Ok(ClientSignal { inner: sender })
}

pub struct LoaderLoopData<B: Backend + 'static> {
    pub display: DisplayHandle,
    pub state: LoaderState<B>,
}

pub fn init_loader(
    event_loop: &EventLoop<LoaderLoopData<WinitBackend>>,
    backend: WinitBackend,
) -> Result<LoaderLoopData<WinitBackend>, Box<dyn std::error::Error>> {
    let display: Display<LoaderState<WinitBackend>> = Display::new()?;
    let dh = display.handle();

    let (client_stream, server_stream) = UnixStream::pair().unwrap();
    let graphics = Arc::new(Mutex::new(Graphics::new()));
    {
        let mut guard = graphics.lock().unwrap();
        xd::test(&mut guard);
    }

    let client_signal = spawn_client_thread(graphics.clone(), client_stream)?;
    let state = LoaderState::init(&dh, backend, event_loop.get_signal(), client_signal);

    display
        .handle()
        .insert_client(server_stream, Arc::new(ClientState::default()))
        .unwrap();

    event_loop.handle().insert_source(
        Generic::new(
            display,
            Interest::READ,
            smithay::reexports::calloop::Mode::Level,
        ),
        |_, display, data| {
            unsafe {
                display.get_mut().dispatch_clients(&mut data.state).unwrap();
            }
            Ok(PostAction::Continue)
        },
    )?;

    let mut data = LoaderLoopData {
        display: dh.clone(),
        state,
    };

    let start_time = std::time::Instant::now();
    let timer = Timer::immediate();

    let output = data.state.backend.create_output();
    let mode = data.state.backend.mode();

    output.create_global::<LoaderState<WinitBackend>>(&dh);
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );

    output.set_preferred(mode);
    data.state.space.map_output(&output, (0, 0));

    let mut output_damage_tracker = OutputDamageTracker::from_output(&output);

    event_loop
        .handle()
        .insert_source(timer, move |_, (), data| {
            let display = &mut data.display;
            let state = &mut data.state;

            let state_ptr: *mut LoaderState<WinitBackend> = state;

            unsafe {
                (*state_ptr)
                    .backend
                    .winit
                    .dispatch_new_events(|event| match event {
                        WinitEvent::Resized {
                            size,
                            scale_factor: _scale_factor,
                        } => {
                            output.change_current_state(
                                Some(Mode {
                                    size,
                                    refresh: 60_000,
                                }),
                                None,
                                None,
                                None,
                            );
                        }
                        WinitEvent::Focus(_) | WinitEvent::Input(_) | WinitEvent::Redraw => {}
                        WinitEvent::CloseRequested => {
                            state.client_signal.stop();
                            state.loop_signal.stop();
                        }
                    });
            }
            {
                let (renderer, mut framebuffer) = state.backend.bind();

                render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(
                    &output,
                    renderer,
                    &mut framebuffer,
                    1.0,
                    0,
                    [&state.space],
                    &[],
                    &mut output_damage_tracker,
                    [0.1, 0.1, 0.1, 1.0],
                )
                .unwrap();
            }

            let size = state.backend.backend.window_size();
            let damage = Rectangle::from_size(size);
            state.backend.backend().submit(Some(&[damage])).unwrap();

            state.space.elements().for_each(|window| {
                window.send_frame(
                    &output,
                    start_time.elapsed(),
                    Some(Duration::ZERO),
                    |_, _| Some(output.clone()),
                );
            });

            state.space.refresh();

            //println!("[{:?}] Flush clients", std::time::Instant::now());
            display.flush_clients().unwrap();
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .unwrap();
    Ok(data)
}
