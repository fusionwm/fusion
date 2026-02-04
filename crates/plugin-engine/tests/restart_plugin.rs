use plugin_engine::{PluginEngine, loader::LoaderConfig, table::CapabilityWriteRules};

use crate::{
    common::{PLUGINS_PATH, initialize, wait_one_second},
    context::call_api::{
        CallApi, CallApiCapProvider, CallApiFactory, PLUGIN, PLUGIN_FILE, check_plugin_clean,
        make_plugin_dirty,
    },
};

mod common;
mod context;

#[test]
fn restart_plugin() -> Result<(), Box<dyn std::error::Error>> {
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

    {
        let plugin_id = engine.get_plugins().first().unwrap().clone();
        engine.restart_plugin(plugin_id);
    }

    wait_one_second(&mut engine);
    check_plugin_clean(&mut engine)?;

    Ok(())
}
