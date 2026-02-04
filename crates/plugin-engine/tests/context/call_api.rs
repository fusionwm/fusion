#![allow(dead_code)]

use std::path::PathBuf;

use plugin_engine::{
    InnerContext, InnerContextFactory, PluginEngine, UntypedPluginBinding,
    context::ExecutionContext,
    impl_untyped_plugin_binding,
    loader::LoaderConfig,
    table::{CapabilityProvider, CapabilityWriteRules},
    wasm::{Component, Linker, Store},
};

use crate::common::{CONFIG_PATH, LOGS_PATH, PLUGINS_PATH};

pub const PLUGIN: &str = "call_api_plugin";
pub const PLUGIN_FILE: &str = "call_api_plugin_1.0.fsp";

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

pub fn check_plugin_clean(
    engine: &mut PluginEngine<CallApi>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = engine.get_single_write_bindings::<TestsApi>("tests-api");
    let mut store = api.store();

    assert!(api.call_get_value(&mut store)? == 0);

    Ok(())
}

pub fn make_plugin_dirty(
    engine: &mut PluginEngine<CallApi>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut api = engine.get_single_write_bindings::<TestsApi>("tests-api");
    let mut store = api.store();

    assert!(api.call_get_value(&mut store)? == 0);

    api.call_add_value(&mut store, 42)?;

    assert!(api.call_get_value(&mut store)? == 42);

    Ok(())
}

pub fn prepare_engine() -> Result<PluginEngine<CallApi>, Box<dyn std::error::Error>> {
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
