mod common;
mod context;

use std::io::Write;

use crate::{
    common::{PLUGINS_PATH, initialize, wait_one_second},
    context::call_api::{
        PLUGIN, PLUGIN_FILE, check_plugin_clean, make_plugin_dirty, prepare_engine,
    },
};

#[test]
fn hot_swap_through_file_watcher_create() -> Result<(), Box<dyn std::error::Error>> {
    initialize(&[PLUGIN]);
    let mut engine = prepare_engine()?;
    engine.load_package(PLUGINS_PATH.path().join(PLUGIN_FILE));

    wait_one_second(&mut engine);

    let plugin_path = PLUGINS_PATH.path().join(PLUGIN_FILE);
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
