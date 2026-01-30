use crate::{
    capabilities::window::fusion::compositor::window_manager::{Host, WindowId},
    context::ExecutionContext,
};

wasmtime::component::bindgen!("window");

impl Host for ExecutionContext {
    fn get_elements(&mut self) -> Vec<WindowId> {
        vec![]
    }

    fn set_window_size(&mut self, window: WindowId, w: u32, h: u32) -> () {}

    fn set_window_pos(&mut self, window: WindowId, x: u32, y: u32) -> () {}
}
