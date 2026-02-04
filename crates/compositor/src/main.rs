#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_lines)]
mod compositor;
mod loader;

use crate::compositor::window::WinitBackend;
use smithay::reexports::calloop::EventLoop;
use tracy_client::Client;

fn setup_logging() {
    use std::io::Write;
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
        .filter_level(log::LevelFilter::Warn)
        .init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Client::start();
    setup_logging();

    //let mut event_loop: EventLoop<LoaderLoopData<WinitBackend>> = EventLoop::try_new()?;
    //let backend = WinitBackend::new().unwrap();
    //let mut loader_data = init_loader(&event_loop, backend)?;

    //event_loop.run(None, &mut loader_data, |_| {})?;

    let mut event_loop: EventLoop<compositor::data::Data<WinitBackend>> = EventLoop::try_new()?;

    let backend = WinitBackend::new().unwrap();
    let mut data = compositor::init_compositor(&event_loop, backend)?;
    event_loop.run(None, &mut data, |_| {})?;

    Ok(())
}
