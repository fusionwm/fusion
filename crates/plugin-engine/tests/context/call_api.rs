#![allow(dead_code)]

use std::path::PathBuf;

use plugin_engine::{
    InnerContext, InnerContextFactory, UntypedPluginBinding,
    context::ExecutionContext,
    impl_untyped_plugin_binding,
    table::CapabilityProvider,
    wasm::{Component, Linker, Store},
};

use crate::common::{CONFIG_PATH, LOGS_PATH, PLUGINS_PATH};

plugin_engine::wasm::bindgen!({
    path: "tests/wit",
    world: "tests-api",
});

pub struct CallApiFactory;
impl InnerContextFactory<CallApi> for CallApiFactory {
    fn generate(&self, _: &[String]) -> CallApi {
        CallApi
    }
}

pub struct CallApi;
impl InnerContext for CallApi {
    type Factory = CallApiFactory;

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

#[allow(dead_code)]
pub struct CallApiCapProvider;
impl CapabilityProvider for CallApiCapProvider {
    type Inner = CallApi;

    fn link_functions(&self, _: &mut Linker<ExecutionContext<Self::Inner>>) {}

    fn create_bindings(
        &self,
        store: &mut Store<ExecutionContext<Self::Inner>>,
        component: &Component,
        linker: &Linker<ExecutionContext<Self::Inner>>,
    ) -> Box<dyn UntypedPluginBinding> {
        Box::new(TestsApi::instantiate(store, component, linker).unwrap())
    }
}

impl_untyped_plugin_binding!(TestsApi);
