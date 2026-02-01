#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_lines)]
mod compositor;
mod loader;

use crate::compositor::window::WinitBackend;
use smithay::reexports::calloop::EventLoop;
use tracy_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Client::start();
    setup_logging();

    //let mut event_loop: EventLoop<LoaderLoopData<WinitBackend>> = EventLoop::try_new()?;
    //let backend = WinitBackend::new().unwrap();
    //let mut loader_data = init_loader(&event_loop, backend)?;

    //event_loop.run(None, &mut loader_data, |_| {})?;

    // Используем EventLoop для обработки событий от разных источников
    let mut event_loop: EventLoop<compositor::data::Data<WinitBackend>> = EventLoop::try_new()?;

    let backend = WinitBackend::new().unwrap();
    let mut data = compositor::init_compositor(&event_loop, backend)?;
    event_loop.run(None, &mut data, |_| {})?;

    Ok(())
}

/*
#[derive(Debug, Clone, Encode, Decode)]
enum SocketCommandResult {
    Done,
    Modules { list: Vec<String> },
}

#[derive(Default, Debug, Copy, Clone, Encode, Decode)]
enum ModuleListFilter {
    #[default]
    All,
    Failed,
    Running,
    Stopped,
}

#[derive(Debug, Copy, Clone, Encode, Decode)]
enum SocketCommand {
    Modules { filter: Option<ModuleListFilter> },
    ReloadModule { id: usize },
}
*/

//TODO
//Custom shaders
//Slider widget
//Round corners
//Realtime compositor loading
//Compositor
//Compositor capabilities
//Unix socket
//Low-level drawing
//Http capabilities

/* Загрузка комопзитора
 *
 * 1) Инициализация загрузочного композитора для отображения процесса загрузки (он содержит только логику загрузки, модулей в нём нет)
 * 2) Загрузка модулей
 * 2.1) Чтение из памяти
 * 2.2) Преобразование в удобный формат
 * 2.3) Загрузка модуля
 * 3) Инициализация основного композитора
 * 4) Инициализация модулей
 * 5) MainLoop
 */
