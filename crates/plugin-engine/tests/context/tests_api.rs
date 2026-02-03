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

pub struct EmptyFactory;
impl InnerContextFactory<Empty> for EmptyFactory {
    fn generate(&self, _: &[String]) -> Empty {
        Empty
    }
}

pub struct Empty;
impl InnerContext for Empty {
    type Factory = EmptyFactory;

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

pub struct TestsApiCapProvider;
impl CapabilityProvider for TestsApiCapProvider {
    type Inner = Empty;

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
