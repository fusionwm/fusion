use crate::{
    Xd,
    fusion::fusion::compositor::window_manager::{
        get_elements, get_output_size, set_window_pos, set_window_size,
    },
};

wit_bindgen::generate!({
    path: "wit-fusion",
});

impl Guest for crate::Xd {
    #[allow(async_fn_in_trait)]
    fn tick() {}

    #[allow(async_fn_in_trait)]
    fn stop() {}

    #[allow(async_fn_in_trait)]
    fn heap_allocated() -> u64 {
        0
    }

    #[allow(async_fn_in_trait)]
    fn heap_capacity() -> u64 {
        0
    }

    #[allow(async_fn_in_trait)]
    fn heap_free() -> u64 {
        0
    }

    #[allow(async_fn_in_trait)]
    fn rearrange_windows() {
        let windows = get_elements();
        let (screen_width, screen_height) = get_output_size();
        let width_per_window = screen_width / windows.len() as u32;

        for (i, window) in windows.iter().enumerate() {
            let x_pos = i as u32 * width_per_window;
            let y_pos = 0;

            set_window_pos(*window, x_pos, y_pos);
            set_window_size(*window, width_per_window, screen_height);
        }

        //TODO send_configure
    }
}

export!(Xd);
