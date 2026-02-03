use std::sync::atomic::{AtomicU8, Ordering};

use crate::Example;

wit_bindgen::generate!({
    path: "../../wit",
    world: "tests-api",
});

static GLOBAL_VALUE: AtomicU8 = AtomicU8::new(0);

impl Guest for Example {
    fn add_value(value: u8) {
        GLOBAL_VALUE.fetch_add(value, Ordering::SeqCst);
    }

    fn get_value() -> u8 {
        GLOBAL_VALUE.load(Ordering::SeqCst)
    }
}

export!(Example);
