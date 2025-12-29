use wayland_server::DisplayHandle;

use crate::compositor::{App, backend::Backend};

// Используется в цикле событий
pub struct Data<B: Backend + 'static> {
    pub display: DisplayHandle,
    pub state: App<B>,
}
