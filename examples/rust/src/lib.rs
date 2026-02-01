use crate::plugin::general::logging::info;

pub mod fusion;

wit_bindgen::generate!({
    path: "../../specs/plugin-base",
    world: "general",
});

pub struct Example;
impl Guest for Example {
    fn init() {
        info("Plugin initialized");
    }
}

export!(Example);
