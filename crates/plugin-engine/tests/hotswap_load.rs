mod common;
mod context;

use crate::{
    common::{PLUGINS_PATH, initialize, wait_one_second},
    context::call_api::{
        PLUGIN, PLUGIN_FILE, check_plugin_clean, make_plugin_dirty, prepare_engine,
    },
};

#[test]
fn hot_swap_through_load() -> Result<(), Box<dyn std::error::Error>> {
    initialize(&[PLUGIN]);

    let mut engine = prepare_engine()?;
    engine.load_package(PLUGINS_PATH.path().join(PLUGIN_FILE));

    wait_one_second(&mut engine);
    make_plugin_dirty(&mut engine)?;

    //Make plugin clean (Through hotswap)
    engine.load_package(PLUGINS_PATH.path().join(PLUGIN_FILE));
    wait_one_second(&mut engine);
    check_plugin_clean(&mut engine)?;

    Ok(())
}
