#![allow(clippy::non_std_lazy_statics)]

use std::path::PathBuf;

use lazy_static::lazy_static;
use plugin_engine::{PluginEngine, loader::LoaderConfig};
use tempfile::TempDir;

use crate::{
    common::{CONFIG_PATH, LOGS_PATH, PLUGINS_PATH, initialize, wait_one_second},
    context::empty::{Empty, EmptyFactory, Paths},
};

mod common;
mod context;

lazy_static! {
    static ref OTHER_PLUGINS_PATH: TempDir = TempDir::new().unwrap();
}

struct OverridePaths;
impl Paths for OverridePaths {
    fn config_path() -> PathBuf {
        CONFIG_PATH.path().to_path_buf()
    }

    fn logs_path() -> PathBuf {
        LOGS_PATH.path().to_path_buf()
    }

    fn plugins_path() -> PathBuf {
        OTHER_PLUGINS_PATH.path().to_path_buf()
    }
}

fn get_packages_count() -> usize {
    let dir = std::fs::read_dir(OTHER_PLUGINS_PATH.path()).unwrap();
    dir.count()
}

#[test]
fn preload_plugins() -> Result<(), Box<dyn std::error::Error>> {
    const PLUGIN: &str = "empty_plugin";
    const PLUGIN_FILE: &str = "empty_plugin_1.0.fsp";

    initialize(&[PLUGIN]);

    std::fs::copy(
        PLUGINS_PATH.path().join(PLUGIN_FILE),
        OTHER_PLUGINS_PATH.path().join(PLUGIN_FILE),
    )?;

    let mut engine = PluginEngine::<Empty<OverridePaths>>::new(
        EmptyFactory,
        LoaderConfig::default()
            .enable_preload(true)
            .manual_loading(false),
    )?;

    wait_one_second(&mut engine);

    assert_eq!(engine.get_plugins().len(), get_packages_count());

    Ok(())
}
