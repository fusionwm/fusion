pub mod backend;
pub mod data;
pub mod grabs;
pub mod input;
pub mod state;
pub mod window;

use std::sync::Arc;
use std::time::Duration;

use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::winit::WinitEvent;
use smithay::desktop::space::render_output;
use smithay::output::Mode;
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::calloop::{EventLoop, Interest, PostAction};
use smithay::reexports::wayland_server::Display;
use smithay::utils::{Rectangle, Transform};
use smithay::wayland::socket::ListeningSocketSource;

use smithay::wayland::compositor::{CompositorClientState, CompositorState};

use wayland_server::backend::{ClientData, ClientId, DisconnectReason};

use crate::compositor::backend::Backend;
use crate::compositor::state::App;
use crate::compositor::window::WinitBackend;

pub fn init_compositor(
    event_loop: &EventLoop<data::Data<WinitBackend>>,
    backend: WinitBackend,
) -> Result<data::Data<WinitBackend>, Box<dyn std::error::Error>> {
    // Структура которая используется для хранения состояния композитора
    // и управления Бэкендом для отправки событий и получения запросов.
    let display: Display<App<WinitBackend>> = Display::new()?;

    // Получаем DisplayHandle который будет использоваться для добавление и получения Wayland клиентов,
    // создания/отключения/удаления глобальных объектов, отправки событий и т.д.
    let dh = display.handle();

    // Wayland ListeningSocket который реализует calloop::EventSource и может быть использован в качестве источника в EventLoop.
    // Клиенты Wayland должны подключаться к этому сокету для получения событий и отправки запросов.
    let socket = ListeningSocketSource::new_auto()?;
    let socket_name = socket.socket_name().to_os_string();

    println!("Socket: {}", socket_name.display());

    unsafe { std::env::set_var("WAYLAND_DISPLAY", &socket_name) };

    // Добавляем сокет Wayland к циклу событий
    // Цикл событий потребляет источник (сокет), затем замыкание, которые производит событие, метаданные и клиентские данные.
    // Событие в этом примере это UnixStream созданный сокетом,
    // без метаданных и клиентских данных которые были определены когда создали переменную event_loop
    event_loop
        .handle()
        .insert_source(socket, |stream, (), data| {
            // Вставляем нового клиента в Display вместе с данными связанными с этим клиентом.
            // Это запустит управление клиентом через UnixStream
            data.display
                .insert_client(stream, Arc::new(ClientState::default()))
                .unwrap();
        })?;

    // Добавляем Display в цикл событий
    // Этот цикл событий может принять обобщенную структуру содержащую файловый дескриптор
    // который будет использоваться для генерации событий. Этот файловый дескриптор создается из winit ниже.
    // Нам только нужно читать (Interest::READ) файловый дескриптор, а Mode::Level будет следить за событиями
    // каждый раз когда цикл событий выполняет опрос.
    event_loop.handle().insert_source(
        Generic::new(
            display,
            Interest::READ,
            smithay::reexports::calloop::Mode::Level,
        ),
        |_, display, data| {
            // Отправка запросов, полученных от клиентов, на обратные вызовы для клиентов.
            // Обратные вызовам, возможно, понадобится доступ к текущему состоянию композитора, поэтому передаём его.
            unsafe {
                display.get_mut().dispatch_clients(&mut data.state).unwrap();
            }

            // Выше ListeningSocketSource обрабатывал цикл обработки событий, указывая PostAction.
            // Здесь мы реализуем наш собственный общий источник событий, поэтому мы должны вернуть
            // PostAction::Continue, чтобы сообщить циклу обработки событий о продолжении прослушивания событий.
            Ok(PostAction::Continue)
        },
    )?;

    // Создаем состояние нашего композитора и передаём все глобальные объекты к которым мы будем обращаться
    let state = App::init(dh.clone(), backend, event_loop.get_signal());

    // Данные хранящиеся в цикле событий, мы должны получать доступ к дисплею и состоянию композитора.
    let mut data = data::Data {
        display: dh.clone(),
        state,
    };

    // Create a timer and start time for the EventLoop.
    // TODO: Use ping for a tighter event loop.
    let start_time = std::time::Instant::now();
    let timer = Timer::immediate();

    let mut output = data.state.backend.create_output();
    let mode = data.state.backend.mode();

    // Клиенты могут получить доступ к глобальным объектам для получения физических свойств и состояния вывода.
    output.create_global::<App<WinitBackend>>(&dh);
    // Устанавливаем состояние для использования winit.
    output.change_current_state(
        // Содержит размер/частоту обновления от winit.
        Some(mode),
        Some(Transform::Flipped180), // OpenGL ES texture?
        None,
        Some((0, 0).into()),
    );

    // Set the prefereed mode to use.
    output.set_preferred(mode);
    // Set the output of a space with coordinates for the upper left corner of the surface.
    data.state.space.map_output(&output, (0, 0));

    // Tracks output for damaged elements allowing for the ability to redraw only what has been damaged.
    let mut output_damage_tracker = OutputDamageTracker::from_output(&output);

    event_loop
        .handle()
        .insert_source(timer, move |_, (), data| {
            let display = &mut data.display;
            let state = &mut data.state;

            //state.backend.dispatch_new_events(&mut output);
            let state_ptr: *mut App<WinitBackend> = state;

            unsafe {
                (*state_ptr)
                    .backend
                    .winit
                    .dispatch_new_events(|event| match event {
                        WinitEvent::Resized { size, scale_factor } => {
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
                        WinitEvent::Focus(_) => {}
                        WinitEvent::Input(input) => state.handle_input_event(input),
                        WinitEvent::CloseRequested => {
                            state.loop_signal.stop();
                        }
                        WinitEvent::Redraw => {
                            state.engine.tick().unwrap();
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

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

impl<B: Backend + 'static> AsMut<CompositorState> for App<B> {
    fn as_mut(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
}
