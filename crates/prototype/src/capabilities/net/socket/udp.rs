use wasmtime::{Caller, Func, Store};

use crate::{
    capabilities::{read_wasm_memory_slice, read_wasm_string, write_wasm_bytes},
    error,
    module::context::ExecutionContext,
};

pub fn net_socket_udp_create(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (bind_addr,): (i32,)| {
            Box::new(async move {
                let Ok(bind_addr) = read_wasm_string(&mut caller, bind_addr).parse() else {
                    return -2;
                };

                caller.data_mut().new_udp(bind_addr).await
            })
        },
    )
}

pub fn net_socket_udp_connect(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (id, remote_addr): (i32, i32)| {
            Box::new(async move {
                let remote_addr = match read_wasm_string(&mut caller, remote_addr).parse() {
                    Ok(remote_addr) => remote_addr,
                    Err(error) => {
                        let logger = caller.data_mut().logger();
                        error!(logger, "[TODO] Failed to parse remote address: {error}");
                        return -1;
                    }
                };

                caller.data_mut().udp_connect(id, remote_addr).await
            })
        },
    )
}

pub fn net_socket_udp_send(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (id, data, len): (i32, i32, i64)| {
            Box::new(async move {
                let data = read_wasm_memory_slice(&mut caller, data, len).to_vec(); //TODO fix
                caller.data_mut().udp_send(id, &data).await
            })
        },
    )
}

pub fn net_socket_udp_recv(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (id, buffer, length): (i32, i32, i64)| {
            Box::new(async move {
                let mut data = vec![0; length as usize];
                let received = caller.data_mut().udp_recv(id, &mut data).await;
                write_wasm_bytes(&mut caller, buffer, &data);
                received
            })
        },
    )
}

pub fn net_socket_udp_shutdown(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, id: i32| -> i32 {
            caller.data_mut().udp_shutdown(id)
        },
    )
}
