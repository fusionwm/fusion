mod api;

use crate::plugin::general::logging::info;

wit_bindgen::generate!({
    path: "../../../../../specs/plugin-base",
    world: "general",
});

pub struct Example;
impl Guest for Example {
    fn init() {
        info("Example plugin initialized");
    }
}

export!(Example);
