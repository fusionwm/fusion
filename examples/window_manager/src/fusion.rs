#![allow(clippy::cast_possible_truncation)]

use std::sync::Mutex;

use crate::{
    WindowManager,
    fusion::fusion::compositor::{
        types::WindowId,
        wm_imports::{get_output_size, send_configure, set_window_pos, set_window_size},
    },
};

wit_bindgen::generate!({
    path: "../../specs/compositor",
    world: "compositor",
});

static WINDOWS: Mutex<Vec<WindowId>> = Mutex::new(Vec::new());

impl exports::fusion::compositor::wm_exports::Guest for crate::WindowManager {
    fn new_toplevel(window: WindowId) {
        WINDOWS.lock().unwrap().push(window);
        Self::rearrange_windows();
    }

    fn toplevel_destroyed(window: WindowId) {
        WINDOWS.lock().unwrap().retain(|&w| w.inner != window.inner);
        Self::rearrange_windows();
    }

    fn rearrange_windows() {
        let windows = WINDOWS.lock().unwrap();
        if windows.is_empty() {
            return;
        }
        let (screen_width, screen_height) = get_output_size();
        let width_per_window = screen_width / windows.len() as u32;

        for (i, window) in windows.iter().enumerate() {
            let window = *window;
            let x_pos = i as u32 * width_per_window;
            let y_pos = 0;

            set_window_pos(window, x_pos, y_pos);
            set_window_size(window, width_per_window, screen_height);
            send_configure(window);
        }
    }
}

impl Guest for crate::WindowManager {
    fn stop() {}
}

export!(WindowManager);
