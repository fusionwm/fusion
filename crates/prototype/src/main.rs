#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

mod capabilities;
mod compositor;
mod module;

use crate::{
    compositor::{data, init_compositor, window::WinitBackend},
    module::{engine::ModuleEngine, loader::ModuleLoader},
};
use bincode::{Decode, Encode};
use graphics::graphics::Graphics;
use log::LevelFilter;
use smithay::reexports::calloop::{self, EventLoop};
use std::{
    intrinsics::breakpoint,
    io::Write,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tokio::task;

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
    let (error_tx, mut error_rx) = tokio::sync::mpsc::channel(16);

    // Используем EventLoop для обработки событий от разных источников
    let mut event_loop: EventLoop<data::Data<WinitBackend>> = EventLoop::try_new()?;

    let backend = WinitBackend::new().unwrap();
    let mut data = init_compositor(&event_loop, backend)?;

    let graphics = Arc::new(Mutex::new(Graphics::new()));

    let module_loader = ModuleLoader::new(error_tx).await?;
    let mut engine = ModuleEngine::new(module_loader, graphics.clone());

    let (executor, scheduler) = calloop::futures::executor()?;
    event_loop
        .handle()
        .insert_source(executor, |(), (), _| {})?;

    let tree = async move {
        tokio::select! {
            _loader = engine.handle_events() => {},
            _main = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                if let Err(error) = engine.tick().await {
                    println!("{error}");
                }
            }
            error = error_rx.recv() => {
                if let Some(error) = error {
                    println!("{error}");
                }
            },
        }
    };

    let mut evl = graphics::EventLoop::new(graphics).unwrap();
    let task = task::spawn_blocking(|| evl.run().unwrap());
    let xd = async move {
        task.await.unwrap();
    };

    scheduler.schedule(tree)?;
    scheduler.schedule(xd)?;

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
