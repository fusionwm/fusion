mod ffi;

use graphics::{
    WindowContent, WindowRoot,
    commands::{DrawCommand, DrawRectCommand},
    reexports::Anchor,
    types::{Argb8888, Bounds, Stroke},
    window::WindowRequest,
};
use wasmtime::{Caller, Func, Store};

use crate::{
    capabilities::{
        graphics::ffi::{CWindowLayer, convert_window_layer_fast},
        read_wasm_memory_slice, read_wasm_string,
    },
    module::context::ExecutionContext,
};

#[derive(Default)]
pub struct ModuleWindow {
    bounds: Bounds,
}

impl WindowContent for ModuleWindow {
    fn desired_size(&self) -> graphics::widget::DesiredSize {
        graphics::widget::DesiredSize::Fill
    }

    fn anchor(&self) -> graphics::reexports::Anchor {
        Anchor::Left
    }

    fn draw<'frame>(&'frame self, out: &mut graphics::commands::CommandBuffer<'frame>) {
        out.push(DrawCommand::Rect(DrawRectCommand::new(
            Bounds::new(self.bounds.position, self.bounds.size),
            Argb8888::GRAY,
            Stroke::NONE,
        )));
    }

    fn layout(&mut self, bounds: graphics::types::Bounds) {
        self.bounds = bounds;
    }

    fn update(&mut self, ctx: &graphics::widget::FrameContext) {}
}

pub struct DynamicWindowRoot {
    request: WindowRequest,
    content: ModuleWindow,
}

impl WindowRoot for DynamicWindowRoot {
    fn request(&self) -> graphics::window::WindowRequest {
        self.request.clone()
    }

    fn setup(&mut self, app: &mut graphics::graphics::Graphics) {}

    fn root_mut(&mut self) -> &mut dyn graphics::WindowContent {
        &mut self.content
    }

    fn root(&self) -> &dyn graphics::WindowContent {
        &self.content
    }
}

pub fn create_window(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, id: i32, layer: i32, width: i32, height: i32| {
            let id = read_wasm_string(&mut caller, id).to_string();
            let layer = {
                unsafe {
                    let c_layer = &*read_wasm_memory_slice(
                        &mut caller,
                        layer,
                        size_of::<CWindowLayer>() as i64,
                    )
                    .as_ptr()
                    .cast();
                    convert_window_layer_fast(c_layer, &mut caller)
                }
            };

            let window = Box::new(DynamicWindowRoot {
                request: WindowRequest::new(id)
                    .with_layer(layer)
                    .with_size(width as u32, height as u32),
                content: ModuleWindow::default(),
            });

            let window_ptr = window.as_ref() as *const _;

            let data = caller.data_mut();
            let mut graphics = data.graphics.lock().unwrap();

            graphics.add_window(window);

            window_ptr as i64
        },
    )
}

macro_rules! pub_wasm_fn {
    ($name:ident, $caller_name:ident, $($p_name:ident : $p_type:ty),* , $content:block) => {
        pub fn $name(store: &mut Store<ExecutionContext>) -> Func {
            Func::wrap(store, |mut $caller_name: Caller<'_, ExecutionContext>, $($p_name : $p_type),*| $content)
        }
    };
}

pub_wasm_fn! {
   destroy_window, caller, window: i64, {
       let window = window as *const DynamicWindowRoot;
       let data = caller.data_mut();
       let mut graphics = data.graphics.lock().unwrap();
       graphics.destroy_window(window);
   }
}

/*

__attribute__((import_module("env")))
__attribute__((import_name("set_window_title")))
void set_window_title(Window window, const char* title);

__attribute__((import_module("env")))
__attribute__((import_name("resize_window")))
void resize_window(Window window, i32 width, i32 height);

__attribute__((import_module("env")))
__attribute__((import_name("move_window")))
void move_window(Window window, i32 x, i32 y);

__attribute__((import_module("env")))
__attribute__((import_name("set_window_visibility")))
void set_window_visibility(Window window, bool visible);

__attribute__((import_module("env")))
__attribute__((import_name("push_draw_command")))
void push_draw_command(Window window, CommandPool pool);
*/
