use http_body_util::Empty;
use log::error;
use wasmtime::{Caller, Func, Store};

use crate::capabilities::{ExecutionContext, read_wasm_string};

pub fn net_http_start_client(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(store, |mut caller: Caller<'_, ExecutionContext>| -> i32 {
        caller.data_mut().create_http_client()
    })
}

pub fn net_http_request(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(store, |mut caller: Caller<'_, ExecutionContext>| -> i32 {
        caller.data_mut().new_http_request()
    })
}

pub fn net_http_set_method(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, id: i32, method: i32| {
            let method = read_wasm_string(&mut caller, method).to_string(); //TODO Fix
            let ctx = caller.data_mut();
            let Some(request) = ctx.get_mut_request(id) else {
                error!("[TODO] Http request '{id}' not found");
                return -1;
            };

            ctx.insert_request(id, request.method(method.as_str()));
            0
        },
    )
}

pub fn net_http_set_uri(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, id: i32, url: i32| {
            let url = read_wasm_string(&mut caller, url).to_string();
            let ctx = caller.data_mut();
            let Some(request) = ctx.get_mut_request(id) else {
                error!("[TODO] Http request '{id}' not found");
                return -1;
            };

            ctx.insert_request(id, request.uri(url.as_str()));
            0
        },
    )
}

/*
pub fn net_http_set_body(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, id: i32, body: i32| {
            let body = read_wasm_string(&mut caller, body).to_string();
            let ctx = caller.data_mut();
            let Some(request) = ctx.get_mut_request(id) else {
                error!("[TODO] Http request '{id}' not found");
                return -1;
            };

            let xd = request.body(body);

            //ctx.insert_request(id, request.body(body.as_str()));
            0
        },
    )
}
*/

pub fn net_http_request_done(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, ExecutionContext>, id: i32| {
            let ctx = caller.data_mut();
            let Some(request) = ctx.get_mut_request(id) else {
                error!("[TODO] Http request '{id}' not found");
                return -1;
            };

            let request = match request.body(Empty::<bytes::Bytes>::new()) {
                Ok(request) => request,
                Err(error) => {
                    error!("Failed to done http request '{id}': {error}");
                    return -1;
                }
            };

            ctx.insert_prepared_request(id, request);

            0
        },
    )
}

pub fn net_http_send_request(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap_async(
        store,
        |mut caller: Caller<'_, ExecutionContext>, (id,): (i32,)| {
            Box::new(async move {
                let ctx = caller.data_mut();
                ctx.send_http_request(id).await
            })
        },
    )
}
