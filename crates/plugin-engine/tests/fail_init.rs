use plugin_engine::PluginEngine;
use plugin_engine::loader::LoaderConfig;

use crate::common::{PLUGINS_PATH, initialize, wait_one_second};
use crate::context::empty::{Empty, EmptyFactory};

mod common;
mod context;

#[test]
fn fail_init() -> Result<(), Box<dyn std::error::Error>> {
    const PLUGIN: &str = "fail_init_plugin";
    const PLUGIN_FILE: &str = "fail_init_plugin_1.0.fsp";
    initialize(&[PLUGIN]);

    let mut engine = PluginEngine::<Empty>::new(
        EmptyFactory,
        LoaderConfig::default()
            .enable_preload(false)
            .manual_loading(false),
    )?;

    engine.load_package(PLUGINS_PATH.path().join(PLUGIN_FILE));

    wait_one_second(&mut engine);

    let module = engine.get_failed_plugins().first().unwrap();
    assert!(module.manifest().name() == "fail_init_plugin");

    Ok(())
}
