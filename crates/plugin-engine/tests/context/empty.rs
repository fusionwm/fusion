#![allow(dead_code)]

use std::path::PathBuf;

use plugin_engine::{InnerContext, InnerContextFactory};

use crate::common::{CONFIG_PATH, LOGS_PATH, PLUGINS_PATH};

pub trait Paths: Send + Sync + 'static {
    fn config_path() -> PathBuf;
    fn logs_path() -> PathBuf;
    fn plugins_path() -> PathBuf;
}

impl Paths for () {
    fn config_path() -> PathBuf {
        CONFIG_PATH.path().to_path_buf()
    }

    fn logs_path() -> PathBuf {
        LOGS_PATH.path().to_path_buf()
    }

    fn plugins_path() -> PathBuf {
        PLUGINS_PATH.path().to_path_buf()
    }
}

pub struct EmptyFactory;
impl<P: Paths> InnerContextFactory<Empty<P>> for EmptyFactory {
    fn generate(&self, _: &[String]) -> Empty<P> {
        Empty::default()
    }
}

pub struct Empty<P = ()> {
    _phantom: std::marker::PhantomData<P>,
}

impl<P> Default for Empty<P> {
    fn default() -> Self {
        Empty {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<P: Paths> InnerContext for Empty<P> {
    type Factory = EmptyFactory;

    fn config_path() -> PathBuf {
        P::config_path()
    }

    fn logs_path() -> PathBuf {
        P::logs_path()
    }

    fn plugins_path() -> PathBuf {
        P::plugins_path()
    }
}
