mod common;
mod context;

use plugin_engine::{PluginEngine, loader::LoaderConfig, table::CapabilityWriteRules};

use crate::{
    common::{PLUGINS_PATH, initialize, wait_one_second},
    context::call_api::{
        CallApi, CallApiCapProvider, CallApiFactory, PLUGIN, PLUGIN_FILE, make_plugin_dirty,
    },
};

#[test]
fn call_api() -> Result<(), Box<dyn std::error::Error>> {
    initialize(&[PLUGIN]);

    let mut engine = PluginEngine::<CallApi>::new(
        CallApiFactory,
        LoaderConfig::default()
            .enable_preload(false)
            .manual_loading(true),
    )?;
    engine.add_capability(
        "tests-api",
        CapabilityWriteRules::SingleWrite,
        CallApiCapProvider,
    );

    engine.load_package(PLUGINS_PATH.path().join(PLUGIN_FILE));

    wait_one_second(&mut engine);
    make_plugin_dirty(&mut engine)?;

    Ok(())
}
