mod state;

use std::{
    os::unix::net::UnixStream,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
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
        FontHandle, TextureHandle, WindowContent, WindowRoot,
        commands::{DrawCommand, DrawTextureCommand},
        fontdue::layout::{CoordinateSystem, Layout, TextStyle},
        glam::Vec2,
        graphics::Graphics,
        reexports::{Anchor, SpecialOptions, TargetMonitor},
        types::{Bounds, Stroke, Texture},
        window::WindowRequest,
    };

    pub struct ModuleWindow {
        bounds: Bounds,
        text: String,
        count: u32,
        layout: Layout,
        font: FontHandle,
        handle: Option<TextureHandle>,
    }

    impl Default for ModuleWindow {
        fn default() -> Self {
            Self {
                bounds: Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
                count: 0,
                layout: Layout::new(CoordinateSystem::PositiveYDown),
                text: String::new(),
                font: FontHandle::default(),
                handle: None,
            }
        }
    }

    impl ModuleWindow {
        fn refresh_layout(&mut self) {
            self.layout.clear();
            self.layout.append(
                &[self.font.as_ref()],
                &TextStyle {
                    text: &self.text,
                    px: 16.0,
                    font_index: 0,
                    user_data: (),
                },
            );
        }
    }

    impl WindowContent for ModuleWindow {
        fn desired_size(&self) -> graphics::widget::DesiredSize {
            graphics::widget::DesiredSize::Fill
        }

        fn anchor(&self) -> graphics::reexports::Anchor {
            graphics::reexports::Anchor::Left
        }

        fn draw<'frame>(&'frame self, out: &mut graphics::commands::CommandBuffer<'frame>) {
            out.push(DrawCommand::Texture(DrawTextureCommand::new(
                self.bounds.clone(),
                Texture::new(graphics::Handle::Texture(self.handle.clone().unwrap())),
                Stroke::NONE,
            )));
        }

        fn layout(&mut self, bounds: graphics::types::Bounds) {
            self.bounds = bounds;
        }

        fn update(&mut self, _ctx: &graphics::widget::FrameContext) {}
    }

    pub struct DynamicWindowRoot {
        request: WindowRequest,
        content: ModuleWindow,
        handle: Option<TextureHandle>,
    }

    impl WindowRoot for DynamicWindowRoot {
        fn request(&self) -> graphics::window::WindowRequest {
            self.request.clone()
        }

        fn setup(&mut self, app: &mut graphics::graphics::Graphics) {
            let handle = Some(
                app.content_manager()
                    .static_load_texture("gachimuchi.jpeg")
                    .unwrap(),
            );

            self.handle = handle.clone();
            self.content.handle = handle.clone();
        }

        fn root_mut(&mut self) -> &mut dyn graphics::WindowContent {
            &mut self.content
        }

        fn root(&self) -> &dyn graphics::WindowContent {
            &self.content
        }
    }

    pub fn test(graphics: &mut Graphics) {
        let window = Box::new(DynamicWindowRoot {
            //request: WindowRequest::new("Test Window")
            //    .with_size(800, 600)
            //    .desktop(DesktopOptions {
            //        title: "Test Window".into(),
            //        resizable: true,
            //        decorations: true,
            //    }),
            request: WindowRequest::new("Test Window")
                .with_size(800, 600)
                .background(SpecialOptions {
                    anchor: Anchor::Bottom,
                    exclusive_zone: 600,
                    target: TargetMonitor::Primary,
                }),
            content: ModuleWindow::default(),
            handle: None,
        });
        graphics.add_window(window);
    }
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

    let state = LoaderState::init(&dh, backend, event_loop.get_signal());

    let (client_stream, server_stream) = UnixStream::pair().unwrap();
    let graphics = Arc::new(Mutex::new(Graphics::new()));
    {
        let mut guard = graphics.lock().unwrap();
        xd::test(&mut guard);
    }
    std::thread::Builder::new()
        .name("Client".into())
        .spawn(|| {
            let mut client = InternalClient::new(graphics, client_stream).unwrap();
            loop {
                if let Err(err) = client.tick(0.0) {
                    eprintln!("Client error: {err}");
                }
            }
        })?;

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

            display.flush_clients().unwrap();
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .unwrap();
    Ok(data)
}
