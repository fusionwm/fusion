#![allow(clippy::non_std_lazy_statics)]

use lazy_static::lazy_static;
use plugin_engine::{InnerContext, PluginEngine};
use std::{path::Path, sync::Once, time::Duration};
use tempfile::TempDir;

fn setup_logging() {
    use log::LevelFilter;
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
        .filter_level(LevelFilter::Off)
        .filter_module("plugin_engine", LevelFilter::Debug)
        .init();
}

fn execute_cargo_fusion(working_dir: &Path) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("cargo-fusion");

    cmd.env_remove("RUSTC_WRAPPER");
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
    cmd.env_remove("CARGO_LLVM_COV");
    cmd.env_remove("__CARGO_LLVM_COV_RUSTC_WRAPPER");
    cmd.env_remove("__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES");
    cmd.env_remove("__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS");

    cmd.arg("build")
        .arg("-o")
        .arg(PLUGINS_PATH.path())
        .current_dir(working_dir)
        .status()?;

    Ok(())
}

fn build_plugins() -> anyhow::Result<()> {
    let plugins = std::env::current_dir()?.join("tests").join("tests_plugins");
    let dir = std::fs::read_dir(plugins)?;
    for entry in dir {
        let entry = entry?;
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        execute_cargo_fusion(&entry.path())?;
    }

    Ok(())
}

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        setup_logging();
        build_plugins().unwrap();
    });
}

lazy_static! {
    pub static ref PLUGINS_PATH: TempDir = tempfile::tempdir().unwrap();
    pub static ref LOGS_PATH: TempDir = tempfile::tempdir().unwrap();
    pub static ref CONFIG_PATH: TempDir = tempfile::tempdir().unwrap();
}

#[allow(clippy::cast_precision_loss)]
pub fn wait_one_second<I: InnerContext>(engine: &mut PluginEngine<I>) {
    const FRAME_COUNT: u64 = 60;
    let frame_time = Duration::from_secs_f32(1.0 / FRAME_COUNT as f32);
    for _ in 0..FRAME_COUNT {
        std::thread::sleep(frame_time);
        engine.load_packages();
    }
}
