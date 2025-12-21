#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

mod capabilities;
mod compositor;
mod module;

use crate::{
    compositor::{data, init_compositor, window::WinitBackend},
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

    // Используем EventLoop для обработки событий от разных источников
    let mut event_loop: EventLoop<data::Data<WinitBackend>> = EventLoop::try_new()?;

    let backend = WinitBackend::new().unwrap();
    let mut data = init_compositor(&event_loop, backend)?;
    event_loop.run(None, &mut data, |_| {})?;

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
//Compositor
//Compositor capabilities
//Unix socket
//Low-level drawing
//Http capabilities
