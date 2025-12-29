use wasmtime::{Caller, Func, Store};

use crate::{capabilities::read_wasm_string, context::ExecutionContext, debug, error, info, warn};

pub fn nms_log_debug(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, msg_ptr: i32| {
            let message = read_wasm_string(&mut caller, msg_ptr).to_string();
            let logger = caller.data_mut().logger();
            debug!(logger, "{message}");
        },
    )
}

pub fn nms_log_info(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, msg_ptr: i32| {
            let message = read_wasm_string(&mut caller, msg_ptr).to_string();
            let logger = caller.data_mut().logger();
            info!(logger, "{message}");
        },
    )
}

pub fn nms_log_warn(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, msg_ptr: i32| {
            let message = read_wasm_string(&mut caller, msg_ptr).to_string();
            let logger = caller.data_mut().logger();
            warn!(logger, "{message}");
        },
    )
}

pub fn nms_log_error(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, msg_ptr: i32| {
            let message = read_wasm_string(&mut caller, msg_ptr).to_string();
            let logger = caller.data_mut().logger();
            error!(logger, "{message}");
        },
    )
}
