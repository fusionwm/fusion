use calloop::LoopSignal;
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_data_device, delegate_layer_shell, delegate_output,
    delegate_seat, delegate_shm, delegate_xdg_shell,
    desktop::{LayerSurface, Space, Window, WindowSurfaceType, layer_map_for_output},
    input::{Seat, SeatHandler, SeatState, keyboard::XkbConfig},
    output::Output,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, get_parent,
            is_sync_subsurface, with_states,
        },
        output::{OutputHandler, OutputManagerState},
        selection::{
            SelectionHandler,
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
        },
        shell::{
            wlr_layer::{
                Layer, LayerSurface as WlrLayerSurface, LayerSurfaceData, WlrLayerShellHandler,
                WlrLayerShellState,
            },
            xdg::{
                PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
                XdgToplevelSurfaceData,
            },
        },
        shm::{ShmHandler, ShmState},
    },
};
use wayland_server::{
    Client, DisplayHandle,
    backend::{ClientData, ClientId, DisconnectReason},
    protocol::{wl_output::WlOutput, wl_seat::WlSeat, wl_surface::WlSurface},
};

use crate::{compositor::backend::Backend, loader::ClientSignal};

pub struct LoaderState<B: Backend + 'static> {
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub seat_state: SeatState<Self>,
    pub shm_state: ShmState,
    pub space: Space<Window>,
    pub output_manager_state: OutputManagerState,
    pub xdg_shell_state: XdgShellState,
    pub wlr_layer_shell_state: WlrLayerShellState,

    pub loop_signal: LoopSignal,
    pub client_signal: ClientSignal,

    pub backend: B,
}

impl<B: Backend + 'static> LoaderState<B> {
    pub fn init(
        dh: &DisplayHandle,
        backend: B,
        loop_signal: LoopSignal,
        client_signal: ClientSignal,
    ) -> Self {
        let compositor_state = CompositorState::new::<Self>(dh);
        let data_device_state = DataDeviceState::new::<Self>(dh);
        let shm_state = ShmState::new::<Self>(dh, vec![]);
        let mut seat_state = SeatState::<Self>::new();
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(dh);
        let xdg_shell_state = XdgShellState::new::<Self>(dh);
        let space = Space::<Window>::default();
        let wlr_layer_shell_state = WlrLayerShellState::new::<Self>(dh);

        let mut seat: Seat<Self> = seat_state.new_wl_seat(dh, "wm_loader");
        seat.add_keyboard(XkbConfig::default(), 500, 500).unwrap();
        // Добавляем указатель (мышь, тачпад и т.д.)
        let pointer_handle = seat.add_pointer();

        seat.add_pointer();

        Self {
            compositor_state,
            data_device_state,
            seat_state,
            shm_state,
            space,
            output_manager_state,
            xdg_shell_state,
            wlr_layer_shell_state,
            loop_signal,
            client_signal,
            backend,
        }
    }
}

delegate_compositor!(@<B: Backend + 'static> LoaderState<B>);
impl<B: Backend + 'static> CompositorHandler for LoaderState<B> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self
                .space
                .elements()
                .find(|w| w.toplevel().unwrap().wl_surface() == &root)
            {
                window.on_commit();
            }
        }

        handle_commit(&self.space, surface);
        handle_layer_shell_commit(&self.space, surface);
    }
}

delegate_output!(@<B: Backend + 'static> LoaderState<B>);
impl<B: Backend + 'static> OutputHandler for LoaderState<B> {}

impl<B: Backend + 'static> BufferHandler for LoaderState<B> {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {}
}

delegate_shm!(@<B: Backend + 'static> LoaderState<B>);
impl<B: Backend + 'static> ShmHandler for LoaderState<B> {
    fn shm_state(&self) -> &smithay::wayland::shm::ShmState {
        &self.shm_state
    }
}

delegate_xdg_shell!(@<B: Backend + 'static> LoaderState<B>);
impl<B: Backend + 'static> XdgShellHandler for LoaderState<B> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        self.space.map_element(window, (0, 0), false);
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
        surface.send_repositioned(token);
    }

    fn move_request(&mut self, _surface: ToplevelSurface, _seat: WlSeat, _serial: Serial) {}

    fn resize_request(
        &mut self,
        _surface: ToplevelSurface,
        _seat: WlSeat,
        _serial: Serial,
        _edges: xdg_toplevel::ResizeEdge,
    ) {
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {}

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}
}

pub fn handle_commit(space: &Space<Window>, surface: &WlSurface) {
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
}

pub fn handle_layer_shell_commit(space: &Space<Window>, surface: &WlSurface) {
    // Handle toplevel commits.
    if let Some(output) = space
        .outputs()
        .find(|o| {
            let map = layer_map_for_output(o);
            map.layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
                .is_some()
        })
        .cloned()
    {
        let mut map = layer_map_for_output(&output);

        map.arrange();

        let layer = map
            .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
            .unwrap();

        let initial_configure_send = with_states(surface, |states| {
            states
                .data_map
                .get::<LayerSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });

        if !initial_configure_send {
            layer.layer_surface().send_configure();
        }
    }
}

delegate_seat!(@<B: Backend + 'static> LoaderState<B>);
impl<B: Backend + 'static> SeatHandler for LoaderState<B> {
    type KeyboardFocus = WlSurface;

    type PointerFocus = WlSurface;

    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut smithay::input::SeatState<Self> {
        &mut self.seat_state
    }
}

#[derive(Default)]
pub struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

impl<B: Backend + 'static> AsMut<CompositorState> for LoaderState<B> {
    fn as_mut(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
}

impl<B: Backend + 'static> SelectionHandler for LoaderState<B> {
    type SelectionUserData = ();
}

impl<B: Backend + 'static> ClientDndGrabHandler for LoaderState<B> {}
impl<B: Backend + 'static> ServerDndGrabHandler for LoaderState<B> {}

delegate_data_device!(@<B: Backend + 'static> LoaderState<B>);
impl<B: Backend + 'static> DataDeviceHandler for LoaderState<B> {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

delegate_layer_shell!(@<B: Backend + 'static> LoaderState<B>);
impl<B: Backend + 'static> WlrLayerShellHandler for LoaderState<B> {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.wlr_layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        wl_output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        let output = wl_output
            .as_ref()
            .and_then(Output::from_resource)
            .unwrap_or_else(|| self.space.outputs().next().unwrap().clone());
        let mut map = layer_map_for_output(&output);
        map.map_layer(&LayerSurface::new(surface, namespace))
            .unwrap();
    }

    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        if let Some((mut map, layer)) = self.space.outputs().find_map(|o| {
            let map = layer_map_for_output(o);
            let layer = map
                .layers()
                .find(|&layer| layer.layer_surface() == &surface)
                .cloned();
            layer.map(|layer| (map, layer))
        }) {
            map.unmap_layer(&layer);
        }
    }
}
