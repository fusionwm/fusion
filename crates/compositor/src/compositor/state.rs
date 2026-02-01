#![allow(unused)]

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use graphics::{InternalClient, graphics::Graphics};
use slotmap::SlotMap;
use smithay::{
    backend::renderer::{element::RenderElement, utils::on_commit_buffer_handler},
    delegate_compositor, delegate_data_device, delegate_output, delegate_seat, delegate_shm,
    delegate_xdg_shell,
    desktop::{
        PopupKind, PopupManager, Space, Window, find_popup_root_surface, get_popup_toplevel_coords,
    },
    input::{
        Seat, SeatHandler, SeatState,
        keyboard::XkbConfig,
        pointer::{Focus, GrabStartData, PointerHandle},
    },
    reexports::{
        calloop::LoopSignal, wayland_protocols::xdg::shell::server::xdg_toplevel,
        x11rb::protocol::xproto::RESIZE_REQUEST_EVENT,
    },
    utils::{Rectangle, Serial},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, get_parent,
            is_sync_subsurface, with_states,
        },
        input_method::InputMethodHandler,
        output::{OutputHandler, OutputManagerState},
        selection::{
            SelectionHandler,
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
        shm::{ShmHandler, ShmState},
    },
};
use wayland_server::{
    Client, DisplayHandle, Resource,
    protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
};

use crate::compositor::{
    ClientState,
    api::{
        CompositorContext, CompositorContextFactory, CompositorGlobals, UnsafeCompositorGlobals,
        general::{Compositor, GeneralCapabilityProvider},
    },
    backend::Backend,
    grabs::{MoveSurfaceGrab, ResizeSurfaceGrab, resize_grab},
};

use module_engine::{
    engine::{InnerContextFactory, ModuleEngine},
    loader::ModuleLoader,
    table::CapabilityWriteRules,
};

pub struct App<B: Backend + 'static> {
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub seat_state: SeatState<Self>,
    pub seat: Seat<Self>,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub xdg_shell_state: XdgShellState,

    pub popups: PopupManager,

    pub loop_signal: LoopSignal,

    pub backend: B,

    pub engine: ModuleEngine<CompositorContext>,

    pub globals: Arc<Mutex<CompositorGlobals>>,
}

impl<B: Backend> App<B> {
    pub fn globals(&self) -> MutexGuard<'_, CompositorGlobals> {
        self.globals.lock().unwrap()
    }
}

impl<B: Backend> App<B> {
    pub fn init(
        dh: &DisplayHandle,
        backend: B,
        loop_signal: LoopSignal,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Композитор нашего композитора
        let compositor_state = CompositorState::new::<Self>(dh);

        // Буфер общей памяти для разделения буферов с клиентами.
        // Например, wl_buffer использует wl_shm для создания общего буфера
        // который будет использоваться композитором для
        // доступа к содержимому поверхности клиента.
        let shm_state = ShmState::new::<Self>(dh, vec![]);

        // Вывод - это пространство которое композитор использует.
        // OutputManagerState говорит wl_output использовать xdg-output extension.
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(dh);

        // Используется для настольных приложений.
        // Определяется два типа Wayland поверхностей клиентов которые могут быть использованы.
        // "toplevel" (для приложений) и "popup" (для диалоговых окон, подсказок и т.д.)
        let xdg_shell_state = XdgShellState::new::<Self>(dh);

        // Seat - группа устройств ввод такие как клавиатуры, мыши и т.д. Это управляет состоянем Seat.
        let mut seat_state = SeatState::<Self>::new();

        // Пространство для назначения окон к нему.
        // Отслеживает окна и выводы.
        // Можно получить доступ через space.element() и space.outputs()
        //let space = Space::<Window>::default();

        // Управляет копированием/вставкой и перетакиванием (drag-and-drop) от устройств ввода
        let data_device_state = DataDeviceState::new::<Self>(dh);

        // Создаём новый Seat из состояния Seat и передаём ему имя.
        let mut seat: Seat<Self> = seat_state.new_wl_seat(dh, "fusion_wm");

        // Добавляем клавиатуру с частоток повтора и задержкой в миллисекундах.
        // Повтор - время повтора, задержка - как должно нужно ждать перез следующим повтором
        seat.add_keyboard(XkbConfig::default(), 500, 500).unwrap();

        // Добавляем указатель (мышь, тачпад и т.д.)
        let pointer_handle = seat.add_pointer();

        let popups = PopupManager::default();

        let mut globals = Arc::new(Mutex::new(CompositorGlobals::new()));

        let factory = {
            CompositorContextFactory {
                globals: globals.clone(),
            }
        };

        // Настройка модулей
        let mut engine = ModuleEngine::new(factory)?;
        engine.add_capability(
            "compositor.window",
            CapabilityWriteRules::SingleWrite,
            GeneralCapabilityProvider,
        );

        Ok(Self {
            compositor_state,
            data_device_state,
            seat_state,
            seat,
            shm_state,
            //space,
            output_manager_state,
            xdg_shell_state,
            popups,
            loop_signal,
            backend,

            engine,
            globals,
        })
    }

    fn unconstrain_popup(&self, popup: &PopupSurface) {
        let Ok(root) = find_popup_root_surface(&PopupKind::Xdg(popup.clone())) else {
            return;
        };
        let space = &self.globals.lock().unwrap().space;
        let Some(window) = space
            .elements()
            .find(|w| w.toplevel().unwrap().wl_surface() == &root)
        else {
            return;
        };

        let output = space.outputs().next().unwrap();
        let output_geo = space.output_geometry(output).unwrap();
        let window_geo = space.element_geometry(window).unwrap();

        // The target geometry for the positioner should be relative to its parent's geometry, so
        // we will compute that here.
        let mut target = output_geo;
        target.loc -= get_popup_toplevel_coords(&PopupKind::Xdg(popup.clone()));
        target.loc -= window_geo.loc;

        popup.with_pending_state(|state| {
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }

    fn rearrange_windows(&mut self) {
        let mut bindings = self
            .engine
            .get_single_write_bindings::<Compositor>("compositor.window");
        let mut store = bindings.store();

        bindings.call_rearrange_windows(&mut store).unwrap();
    }

    /*
    fn rearrange_windows(&mut self) {
        let all_windows: Vec<_> = self.space.elements().cloned().collect();
        let count = all_windows.len();
        if count == 0 {
            return;
        }

        // 1. Берем первый попавшийся output из пространства
        let (screen_width, screen_height) = if let Some(output) = self.space.outputs().next() {
            // 2. Получаем текущее состояние (resolution, scale и т.д.)
            let current_mode = output.current_mode().expect("Output has no mode set");

            // Физическое разрешение (например, 1920x1080)
            let physical_size = current_mode.size;

            // 3. Чтобы тайлинг работал корректно с HiDPI, лучше брать логический размер
            let geometry = self
                .space
                .output_geometry(output)
                .expect("Output not in space");
            let screen_width = geometry.size.w;
            let screen_height = geometry.size.h;

            //println!("Размер экрана: {}x{}", screen_width, screen_height);
            // Допустим, мы делим экран вертикально на равные части
            (screen_width, screen_height)
        } else {
            panic!("TODO!")
        };

        let width_per_window = screen_width / count as i32;
        println!("Width per window: {width_per_window}");

        for (i, window) in all_windows.into_iter().enumerate() {
            let x_pos = i as i32 * width_per_window;
            let y_pos = 0;

            // Обновляем позицию окна
            self.space.map_element(window.clone(), (x_pos, y_pos), true);

            // ВАЖНО: Отправляем клиенту команду изменить размер самого окна!
            // Без этого приложение будет думать, что оно все еще 0x0 или другого размера.
            let surface = window.toplevel().unwrap();
            surface.with_pending_state(|state| {
                state.size = Some((width_per_window, 1080).into());
            });
            surface.send_configure();
        }
    }
    */
}

delegate_seat!(@<B: Backend + 'static> App<B>);
impl<B: Backend + 'static> SeatHandler for App<B> {
    type KeyboardFocus = WlSurface;

    type PointerFocus = WlSurface;

    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut smithay::input::SeatState<Self> {
        &mut self.seat_state
    }
}

delegate_compositor!(@<B: Backend + 'static> App<B>);
impl<B: Backend + 'static> CompositorHandler for App<B> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        let space = &mut self.globals.lock().unwrap().space;
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = space
                .elements()
                .find(|w| w.toplevel().unwrap().wl_surface() == &root)
            {
                window.on_commit();
            }
        }

        handle_commit(&mut self.popups, space, surface);
        resize_grab::handle_commit(space, surface);
    }
}

delegate_output!(@<B: Backend + 'static> App<B>);
impl<B: Backend + 'static> OutputHandler for App<B> {}

impl<B: Backend + 'static> BufferHandler for App<B> {
    fn buffer_destroyed(&mut self, buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {}
}

delegate_shm!(@<B: Backend + 'static> App<B>);
impl<B: Backend + 'static> ShmHandler for App<B> {
    fn shm_state(&self) -> &smithay::wayland::shm::ShmState {
        &self.shm_state
    }
}

delegate_xdg_shell!(@<B: Backend + 'static> App<B>);
impl<B: Backend + 'static> XdgShellHandler for App<B> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        {
            let mut globals = self.globals.lock().unwrap();
            let space = &mut globals.space;

            space.map_element(window.clone(), (0, 0), false);
            globals.mapped_windows.insert(window);
        }

        self.rearrange_windows();
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        self.unconstrain_popup(&surface);
        let _ = self.popups.track_popup(PopupKind::Xdg(surface));
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        surface.with_pending_state(|state| {
            let geometry = positioner.get_geometry();
            state.geometry = geometry;
            state.positioner = positioner;
        });
        self.unconstrain_popup(&surface);
        surface.send_repositioned(token);
    }

    fn move_request(&mut self, surface: ToplevelSurface, seat: WlSeat, serial: Serial) {
        let seat = Seat::from_resource(&seat).unwrap();
        let wl_surface = surface.wl_surface();

        if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
            let pointer = seat.get_pointer().unwrap();

            let grab = {
                let space = &mut self.globals.lock().unwrap().space;
                let window = space
                    .elements()
                    .find(|w| w.toplevel().unwrap().wl_surface() == wl_surface)
                    .unwrap()
                    .clone();
                let initial_window_location = space.element_location(&window).unwrap();

                MoveSurfaceGrab {
                    start_data,
                    window,
                    initial_window_location,
                }
            };

            pointer.set_grab(self, grab, serial, Focus::Clear);
        }
    }

    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        seat: WlSeat,
        serial: Serial,
        edges: xdg_toplevel::ResizeEdge,
    ) {
        let seat = Seat::from_resource(&seat).unwrap();

        let wl_surface = surface.wl_surface();

        if let Some(start_data) = check_grab(&seat, wl_surface, serial) {
            let pointer = seat.get_pointer().unwrap();

            let grab = {
                let space = &self.globals.lock().unwrap().space;

                let window = space
                    .elements()
                    .find(|w| w.toplevel().unwrap().wl_surface() == wl_surface)
                    .unwrap()
                    .clone();
                let initial_window_location = space.element_location(&window).unwrap();
                let initial_window_size = window.geometry().size;

                surface.with_pending_state(|state| {
                    state.states.set(xdg_toplevel::State::Resizing);
                });

                surface.send_pending_configure();

                ResizeSurfaceGrab::start(
                    start_data,
                    window,
                    edges.into(),
                    Rectangle::new(initial_window_location, initial_window_size),
                )
            };

            pointer.set_grab(self, grab, serial, Focus::Clear);
        }
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        // TODO popup grabs
    }
}

impl<B: Backend + 'static> SelectionHandler for App<B> {
    type SelectionUserData = ();
}

impl<B: Backend + 'static> ClientDndGrabHandler for App<B> {}
impl<B: Backend + 'static> ServerDndGrabHandler for App<B> {}

delegate_data_device!(@<B: Backend + 'static> App<B>);
impl<B: Backend + 'static> DataDeviceHandler for App<B> {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

pub fn handle_commit(popups: &mut PopupManager, space: &Space<Window>, surface: &WlSurface) {
    // Handle toplevel commits.
    if let Some(window) = space
        .elements()
        .find(|w| w.toplevel().unwrap().wl_surface() == surface)
        .cloned()
    {
        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });

        if !initial_configure_sent {
            window.toplevel().unwrap().send_configure();
        }
    }

    // Handle popup commits.
    popups.commit(surface);
    if let Some(popup) = popups.find_popup(surface) {
        match popup {
            PopupKind::Xdg(ref xdg) => {
                if !xdg.is_initial_configure_sent() {
                    // NOTE: This should never fail as the initial configure is always
                    // allowed.
                    xdg.send_configure().expect("initial configure failed");
                }
            }
            PopupKind::InputMethod(ref _input_method) => {}
        }
    }
}

fn check_grab<B: Backend + 'static>(
    seat: &Seat<App<B>>,
    surface: &WlSurface,
    serial: Serial,
) -> Option<GrabStartData<App<B>>> {
    let pointer = seat.get_pointer()?;

    // Check that this surface has a click grab.
    if !pointer.has_grab(serial) {
        return None;
    }

    let start_data = pointer.grab_start_data()?;

    let (focus, _) = start_data.focus.as_ref()?;
    // If the focus was for a different surface, ignore the request.
    if !focus.id().same_client_as(&surface.id()) {
        return None;
    }

    Some(start_data)
}
