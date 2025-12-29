use wasmtime::{Caller, Func, Store};

use crate::capabilities::ExecutionContext;

pub fn nms_audio_set_volume(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(
        store,
        |_caller: Caller<'_, ExecutionContext>, value: i32| {
            println!("Set master volume: {value}");
        },
    )
}

pub fn nms_audio_mute(store: &mut Store<ExecutionContext>) -> Func {
    Func::wrap(store, |enable: i32| {
        println!("Mute: {enable}");
    })
}
