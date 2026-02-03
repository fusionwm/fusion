use plugin_engine::{PluginEngine, loader::LoaderConfig, table::CapabilityWriteRules};

use crate::{
    common::{PLUGINS_PATH, initialize, wait_one_second},
    context::call_api::{CallApi, CallApiCapProvider, CallApiFactory, TestsApi},
};

mod common;
mod context;

#[test]
fn restart_plugin() -> Result<(), Box<dyn std::error::Error>> {
    initialize();

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

    engine.load_package(PLUGINS_PATH.path().join("plugin_1.0.fsp"));

    wait_one_second(&mut engine);

    {
        let mut api = engine.get_single_write_bindings::<TestsApi>("tests-api");
        let mut store = api.store();

        assert!(api.call_get_value(&mut store)? == 0);
        api.call_add_value(&mut store, 42)?;

        assert!(api.call_get_value(&mut store)? == 42);
    }

    {
        let plugin_id = engine.get_plugins().first().unwrap().clone();
        engine.restart_plugin(plugin_id);
    }

    wait_one_second(&mut engine);

    {
        let mut api = engine.get_single_write_bindings::<TestsApi>("tests-api");
        let mut store = api.store();

        assert!(api.call_get_value(&mut store)? == 0);
    }

    Ok(())
}
