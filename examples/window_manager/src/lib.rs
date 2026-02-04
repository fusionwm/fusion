use crate::plugin::general::logging::info;

pub mod fusion;

wit_bindgen::generate!({
    path: "../../specs/plugin-base",
    world: "general",
});

pub struct WindowManager;
impl Guest for WindowManager {
    fn init() {
        info("Plugin initialized");
    }
}

export!(WindowManager);
