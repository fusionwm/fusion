#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_lines)]
mod capabilities;
mod compositor;
mod loader;
mod module;

use crate::{
    compositor::window::WinitBackend,
    loader::{LoaderLoopData, init_loader},
    module::loader::ModuleLoaderError,
};
use bincode::{Decode, Encode};
use log::LevelFilter;
use smithay::reexports::calloop::EventLoop;
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    PathDoesntExist(String),

    #[error("{0}")]
    IO(#[from] std::io::Error),

    #[error("{0}")]
    TOML(#[from] toml::de::Error),

    #[error("{0}")]
    Module(#[from] wasmtime::Error),

    #[error("{0}")]
    ModuleLoader(#[from] ModuleLoaderError),
}

fn setup_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter_level(LevelFilter::Warn)
        .filter_module("prototype", LevelFilter::Debug)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();

    let mut event_loop: EventLoop<LoaderLoopData<WinitBackend>> = EventLoop::try_new()?;
    let backend = WinitBackend::new().unwrap();
    let mut loader_data = init_loader(&event_loop, backend)?;

    event_loop.run(None, &mut loader_data, |_| {})?;

    // Используем EventLoop для обработки событий от разных источников
    //let mut event_loop: EventLoop<data::Data<WinitBackend>> = EventLoop::try_new()?;

    //let backend = WinitBackend::new().unwrap();
    //let mut data = init_compositor(&event_loop, backend)?;
    //event_loop.run(None, &mut data, |_| {})?;

    Ok(())
}

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

//TODO
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
