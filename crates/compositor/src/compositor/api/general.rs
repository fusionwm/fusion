use std::any::TypeId;

use plugin_engine::{
    context::ExecutionContext,
    engine::UntypedPluginBinding,
    table::CapabilityProvider,
    wasm::{Linker, bindgen},
};
use slotmap::KeyData;

use crate::compositor::api::{
    CompositorContext, CompositorGlobals, PluginContextType, WindowKey,
    general::fusion::compositor::window_manager::{self, WindowId},
};

bindgen!("compositor");

pub struct GeneralCapabilityProvider;
impl CapabilityProvider for GeneralCapabilityProvider {
    type Inner = CompositorContext;

    fn link_functions(&self, linker: &mut Linker<ExecutionContext<Self::Inner>>) {
        Compositor::add_to_linker::<_, CompositorContext>(linker, |state| &mut state.inner)
            .unwrap();
    }

    fn create_bindings(
        &self,
        store: &mut wasmtime::Store<ExecutionContext<Self::Inner>>,
        component: &wasmtime::component::Component,
        linker: &Linker<ExecutionContext<Self::Inner>>,
    ) -> Box<dyn plugin_engine::engine::UntypedPluginBinding> {
        Box::new(Compositor::instantiate(&mut *store, component, linker).unwrap())
    }
}

impl UntypedPluginBinding for Compositor {
    fn type_id(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CompositorContext {
    #[inline]
    fn compositor(&self) -> std::sync::MutexGuard<'_, CompositorGlobals> {
        self.variants
            .get(&PluginContextType::Compositor)
            .unwrap()
            .compositor()
    }

    #[inline]
    fn compositor_mut(&mut self) -> std::sync::MutexGuard<'_, CompositorGlobals> {
        self.variants
            .get_mut(&PluginContextType::Compositor)
            .unwrap()
            .compositor_mut()
    }
}

impl window_manager::Host for CompositorContext {
    fn get_elements(&mut self) -> Vec<WindowId> {
        let mut compositor = self.compositor_mut();
        compositor
            .mapped_windows
            .keys()
            .map(|key| WindowId {
                inner: key.0.as_ffi(),
            })
            .collect()
    }

    fn set_window_size(&mut self, window: WindowId, width: u32, height: u32) {
        let compositor = self.compositor_mut();
        let window = compositor
            .mapped_windows
            .get(WindowKey(KeyData::from_ffi(window.inner)))
            .unwrap();

        let surface = window.toplevel().unwrap();
        surface.with_pending_state(|state| {
            state.size = Some((width as i32, height as i32).into());
        });
        surface.send_configure();
    }

    fn set_window_pos(&mut self, window: WindowId, x: u32, y: u32) {
        let mut compositor = self.compositor_mut();
        let window = compositor
            .mapped_windows
            .get(WindowKey(KeyData::from_ffi(window.inner)))
            .unwrap()
            .clone();

        compositor
            .space
            .map_element(window, (x as i32, y as i32), true);
    }

    fn get_output_size(&mut self) -> (u32, u32) {
        let compositor = self.compositor();
        // 1. Берем первый попавшийся output из пространства
        if let Some(output) = compositor.space.outputs().next() {
            // 2. Получаем текущее состояние (resolution, scale и т.д.)
            //let current_mode = output.current_mode().expect("Output has no mode set");

            // Физическое разрешение (например, 1920x1080)
            //let physical_size = current_mode.size;

            // 3. Чтобы тайлинг работал корректно с HiDPI, лучше брать логический размер
            let geometry = compositor
                .space
                .output_geometry(output)
                .expect("Output not in space");
            let screen_width = geometry.size.w as u32;
            let screen_height = geometry.size.h as u32;

            (screen_width, screen_height)
        } else {
            panic!("TODO!")
        }
    }
}
