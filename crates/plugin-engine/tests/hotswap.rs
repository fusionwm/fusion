mod common;
mod context;

use std::io::Write;

use plugin_engine::{PluginEngine, loader::LoaderConfig, table::CapabilityWriteRules};

use crate::{
    common::{PLUGINS_PATH, initialize, wait_one_second},
    context::call_api::{CallApi, CallApiCapProvider, CallApiFactory, TestsApi},
};

fn make_plugin_dirty(engine: &mut PluginEngine<CallApi>) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = engine.get_single_write_bindings::<TestsApi>("tests-api");
    let mut store = api.store();

    assert!(api.call_get_value(&mut store)? == 0);
    api.call_add_value(&mut store, 42)?;

    assert!(api.call_get_value(&mut store)? == 42);

    Ok(())
}

fn check_plugin_clean(
    engine: &mut PluginEngine<CallApi>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = engine.get_single_write_bindings::<TestsApi>("tests-api");
    let mut store = api.store();

    assert!(api.call_get_value(&mut store)? == 0);

    Ok(())
}

fn prepare_engine() -> Result<PluginEngine<CallApi>, Box<dyn std::error::Error>> {
    let mut engine = PluginEngine::<CallApi>::new(
        CallApiFactory,
        LoaderConfig::default()
            .enable_preload(false)
            .manual_loading(false),
    )?;
    engine.add_capability(
        "tests-api",
        CapabilityWriteRules::SingleWrite,
        CallApiCapProvider,
    );
    Ok(engine)
}

#[test]
fn hot_swap_through_load() -> Result<(), Box<dyn std::error::Error>> {
    initialize();
    let mut engine = prepare_engine()?;
    engine.load_package(PLUGINS_PATH.path().join("call_api_plugin_1.0.fsp"));

    wait_one_second(&mut engine);
    make_plugin_dirty(&mut engine)?;

    //Make plugin clean (Through hotswap)
    engine.load_package(PLUGINS_PATH.path().join("call_api_plugin_1.0.fsp"));
    wait_one_second(&mut engine);
    check_plugin_clean(&mut engine)?;

    Ok(())
}

#[test]
fn hot_swap_through_file_watcher_rename() -> Result<(), Box<dyn std::error::Error>> {
    initialize();
    let mut engine = prepare_engine()?;
    engine.load_package(PLUGINS_PATH.path().join("call_api_plugin_1.0.fsp"));

    wait_one_second(&mut engine);

    let plugin_path = PLUGINS_PATH.path().join("call_api_plugin_1.0.fsp");
    let temp = tempfile::tempdir()?;
    let new_plugin_path = temp.path().join("call_api_plugin_1.0.fsp");
    std::fs::rename(plugin_path.clone(), new_plugin_path.clone())?;

    make_plugin_dirty(&mut engine)?;

    //Make plugin clean (Through file watcher)
    std::fs::rename(new_plugin_path, plugin_path)?;
    wait_one_second(&mut engine);
    check_plugin_clean(&mut engine)?;

    Ok(())
}

#[test]
fn hot_swap_through_file_watcher_create() -> Result<(), Box<dyn std::error::Error>> {
    initialize();
    let mut engine = prepare_engine()?;
    engine.load_package(PLUGINS_PATH.path().join("call_api_plugin_1.0.fsp"));

    wait_one_second(&mut engine);

    let plugin_path = PLUGINS_PATH.path().join("call_api_plugin_1.0.fsp");
    let bytes = std::fs::read(&plugin_path)?;
    std::fs::remove_file(&plugin_path)?;

    make_plugin_dirty(&mut engine)?;

    //Make plugin clean (Through file watcher)
    let mut file = std::fs::File::create(&plugin_path)?;
    file.write_all(&bytes)?;

    wait_one_second(&mut engine);
    check_plugin_clean(&mut engine)?;

    Ok(())
}
