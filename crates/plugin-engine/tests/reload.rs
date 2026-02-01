#![allow(clippy::non_std_lazy_statics)]

use lazy_static::lazy_static;
use std::path::PathBuf;

use plugin_engine::{InnerContext, InnerContextFactory, PluginEngine};

lazy_static! {
    static ref PLUGINS_PATH: PathBuf = {
        let mut tempdir = tempfile::tempdir().unwrap();
        tempdir.disable_cleanup(true);
        tempdir.keep()
    };
    static ref LOGS_PATH: PathBuf = {
        let mut tempdir = tempfile::tempdir().unwrap();
        tempdir.disable_cleanup(true);
        tempdir.keep()
    };
    static ref CONFIG_PATH: PathBuf = {
        let mut tempdir = tempfile::tempdir().unwrap();
        tempdir.disable_cleanup(true);
        tempdir.keep()
    };
}

struct EmptyFactory;
impl InnerContextFactory<Empty> for EmptyFactory {
    fn generate(&self, _: &[String]) -> Empty {
        Empty
    }
}

struct Empty;
impl InnerContext for Empty {
    type Factory = EmptyFactory;

    fn config_path() -> std::path::PathBuf {
        CONFIG_PATH.clone()
    }

    fn logs_path() -> std::path::PathBuf {
        LOGS_PATH.clone()
    }

    fn plugins_path() -> std::path::PathBuf {
        PLUGINS_PATH.clone()
    }
}

#[test]
fn reload_plugin() -> Result<(), Box<dyn std::error::Error>> {
    let engine = PluginEngine::<Empty>::new(EmptyFactory)?;
    Ok(())
}
