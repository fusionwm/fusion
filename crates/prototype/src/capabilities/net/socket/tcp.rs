use wasmtime::{Caller, Func, Store};

use crate::{
    capabilities::{read_wasm_memory_slice, read_wasm_string, write_wasm_bytes},
    module::context::ExecutionContext,
};

pub fn net_socket_tcp_create(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(store, |mut caller: Caller<'_, ExecutionContext>| -> i32 {
        caller.data_mut().new_tcp()
    })
}

pub fn net_socket_tcp_connect(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (id, addr_ptr)| {
            Box::new(async move {
                let Ok(addr) = read_wasm_string(&mut caller, addr_ptr).parse() else {
                    return -1;
                };

                caller.data_mut().tcp_connect(id, addr).await;
                0
            })
        },
    )
}

pub fn net_socket_tcp_send(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (id, data, length): (i32, i32, i64)| {
            Box::new(async move {
                let data = read_wasm_memory_slice(&mut caller, data, length).to_vec(); //TODO fix
                caller.data_mut().tcp_send(id, &data).await;

                0
            })
        },
    )
}

pub fn net_socket_tcp_recv(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (id, buffer, length): (i32, i32, i64)| {
            Box::new(async move {
                let mut data = vec![0; length as usize];
                let received = caller.data_mut().tcp_recv(id, &mut data).await;
                write_wasm_bytes(&mut caller, buffer, &data);
                received
            })
        },
    )
}

pub fn net_socket_tcp_shutdown(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (id,): (i32,)| {
            Box::new(async move {
                caller.data_mut().tcp_shutdown(id).await;
                0
            })
        },
    )
}
