pub mod general;
pub mod graphics;
pub mod net;
pub mod system;

use crate::{
    capabilities::{
        general::log::{nms_log_debug, nms_log_error, nms_log_info, nms_log_warn},
        graphics::{create_window, destroy_window},
        net::{
            http::{
                net_http_request, net_http_request_done, net_http_send_request,
                net_http_set_method, net_http_set_uri, net_http_start_client,
            },
            socket::{
                tcp::{
                    net_socket_tcp_connect, net_socket_tcp_create, net_socket_tcp_recv,
                    net_socket_tcp_send, net_socket_tcp_shutdown,
                },
                udp::{
                    net_socket_udp_connect, net_socket_udp_create, net_socket_udp_recv,
                    net_socket_udp_send, net_socket_udp_shutdown,
                },
            },
        },
        system::audio::{nms_audio_mute, nms_audio_set_volume},
    },
    module::{
        context::ExecutionContext,
        ffi::{FfiValue, ModuleAllocator},
    },
};
use wasmtime::{Caller, Extern, Func, ImportType, Store};

fn read_wasm_memory_slice<'a>(
    caller: &'a mut Caller<'_, ExecutionContext>,
    ptr: i32,
    length: i64,
) -> &'a [u8] {
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .expect("Memory export not found");
    let mem = memory.data(caller);
    let offset = ptr as usize;
    &mem[offset..offset + length as usize]
}

fn get_wasm_memory<'a>(caller: &'a mut Caller<'_, ExecutionContext>) -> &'a [u8] {
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .expect("Memory export not found");
    memory.data(caller)
}

fn read_wasm_string<'a>(caller: &'a mut Caller<'_, ExecutionContext>, ptr: i32) -> &'a str {
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .expect("Memory export not found");
    let mem = memory.data(caller);
    let offset = ptr as usize;
    let end = mem[offset..]
        .iter()
        .position(|&b| b == 0)
        .expect("missing null terminator");
    std::str::from_utf8(&mem[offset..offset + end]).expect("invalid UTF-8")
}

fn write_wasm_bytes(caller: &mut Caller<'_, ExecutionContext>, ptr: i32, data: &[u8]) -> i32 {
    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .expect("Memory export not found");
    let mem = memory.data_mut(caller);
    let offset = ptr as usize;
    mem[offset..offset + data.len()].copy_from_slice(data);
    data.len() as i32
}

fn nms_config_get(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, path_ptr: i32| -> i32 {
            let memory = caller
                .get_export("memory")
                .and_then(Extern::into_memory)
                .expect("memory export not found");

            let mem = memory.data(&caller);
            let offset = path_ptr as usize;
            let end = mem[offset..]
                .iter()
                .position(|&b| b == 0)
                .expect("missing null terminator");
            let slice = &mem[offset..offset + end];
            let path = std::str::from_utf8(slice).expect("invalid UTF-8");

            let config = caller.data().config();
            let value = config.get_value(path).unwrap();

            let alloc_func = caller
                .get_export("module_alloc")
                .and_then(Extern::into_func)
                .unwrap();
            let mut allocator = ModuleAllocator::new(alloc_func, &mut caller, memory);
            let ffi_value = FfiValue::new(value, &mut allocator);
            let ptr = allocator.alloc_bytes(&ffi_value);
            ptr as i32
        },
    )
}

macro_rules! register_imports {
    (
        $import:expr,
        $store:ident,
        $(
            capability: $cap:expr => {
                $( $name:literal => $func:ident ),* $(,)?
            }
        )*
    ) => {
        match $import {
        $(
            $(
                $name if has_capability($store, $cap) => $func($store),
            )*
        )*
        _ => panic!("Unknown import '{:?}'", $import),
        }.into()
    };
}

pub fn get_imports<'module>(
    imports: impl ExactSizeIterator<Item = ImportType<'module>> + 'module,
    store: &mut Store<ExecutionContext>,
) -> Vec<Extern> {
    let mut vec = vec![];
    imports.for_each(|import| {
        vec.push(register_imports!(
            import.name(), store,
            capability: "general" => {
                "nms_log_debug" => nms_log_debug,
                "nms_log_info" => nms_log_info,
                "nms_log_warn" => nms_log_warn,
                "nms_log_error" => nms_log_error,
                "nms_config_get" => nms_config_get,
            }
            capability: "system.audio" => {
                "nms_audio_mute" => nms_audio_mute,
                "nms_audio_set_volume" => nms_audio_set_volume,
            }
            capability: "net.socket.tcp" => {
                "nms_net_socket_tcp_create" => net_socket_tcp_create,
                "nms_net_socket_tcp_connect" => net_socket_tcp_connect,
                "nms_net_socket_tcp_send" => net_socket_tcp_send,
                "nms_net_socket_tcp_recv" => net_socket_tcp_recv,
                "nms_net_socket_tcp_shutdown" => net_socket_tcp_shutdown,
            }
            capability: "net.socket.udp" => {
                "nms_net_socket_udp_create" => net_socket_udp_create,
                "nms_net_socket_udp_connect" => net_socket_udp_connect,
                "nms_net_socket_udp_send" => net_socket_udp_send,
                "nms_net_socket_udp_recv" => net_socket_udp_recv,
                "nms_net_socket_udp_shutdown" => net_socket_udp_shutdown,
            }
            capability: "net.http" => {
                "nms_net_http_start_client" => net_http_start_client,
                "nms_net_http_request" => net_http_request,
                "nms_net_set_method" => net_http_set_method,
                "nms_net_set_uri" => net_http_set_uri,
                "nms_net_request_done" => net_http_request_done,
                "nms_net_http_send_request" => net_http_send_request,
            }
            capability: "general.graphics" => {
                "create_window" => create_window,
                "destroy_window" => destroy_window,
            }
        ));
    });

    vec
}

#[inline]
fn has_capability(store: &mut Store<ExecutionContext>, capability: &str) -> bool {
    store.data().has_capability(capability)
}
