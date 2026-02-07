#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_lines)]
mod compositor;

use crate::compositor::{
    udev::{UdevData, init_udev},
    window::WinitBackend,
};
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
        //.filter_level(log::LevelFilter::Debug)
        .init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Client::start();
    setup_logging();

    run_udev()?;

    Ok(())
}

#[allow(unused)]
fn run_winit() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop: EventLoop<compositor::data::Data<WinitBackend>> = EventLoop::try_new()?;

    let backend = WinitBackend::new().unwrap();
    let mut data =
        compositor::init_compositor(event_loop.handle(), event_loop.get_signal(), backend)?;
    event_loop.run(None, &mut data, |_| {})?;
    Ok(())
}

fn run_udev() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop: EventLoop<compositor::data::Data<UdevData>> = EventLoop::try_new()?;

    let backend = UdevData::init(event_loop.handle());
    let mut data =
        compositor::init_compositor(event_loop.handle(), event_loop.get_signal(), backend)?;

    init_udev(&mut data.state);

    event_loop.run(None, &mut data, |data| {
        data.state.render_all();
        data.state.handle_socket();
        data.state.engine.load_packages();
    })?;
    Ok(())
}
