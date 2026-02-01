use crate::plugin::general::logging::error;

pub mod fusion;

wit_bindgen::generate!({
    path: "wit-plugin",
});

pub struct Xd;
impl Guest for Xd {
    fn init() {
        error("Plugin initialized");
    }
}

export!(Xd);
