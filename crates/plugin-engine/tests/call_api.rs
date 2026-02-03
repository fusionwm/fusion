#![allow(clippy::non_std_lazy_statics)]

use lazy_static::lazy_static;
use std::{path::PathBuf, time::Duration};

use plugin_engine::{
    InnerContext, InnerContextFactory, PluginEngine, UntypedPluginBinding,
    table::{CapabilityProvider, CapabilityWriteRules},
};

lazy_static! {
    static ref PLUGINS_PATH: PathBuf = {
        let working_dir = std::env::current_dir().unwrap();
        working_dir.join("tests").join("packed")
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

fn setup_logging() {
    use log::LevelFilter;
    use std::io::Write;

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter_level(LevelFilter::Info)
        .init();
}

plugin_engine::wasm::bindgen!({
    path: "tests/wit",
    world: "tests-api",
});

struct TestsApiCapProvider;
impl CapabilityProvider for TestsApiCapProvider {
    type Inner = Empty;

    fn link_functions(
        &self,
        _: &mut wasmtime::component::Linker<plugin_engine::context::ExecutionContext<Self::Inner>>,
    ) {
    }

    fn create_bindings(
        &self,
        store: &mut wasmtime::Store<plugin_engine::context::ExecutionContext<Self::Inner>>,
        component: &wasmtime::component::Component,
        linker: &wasmtime::component::Linker<plugin_engine::context::ExecutionContext<Self::Inner>>,
    ) -> Box<dyn plugin_engine::UntypedPluginBinding> {
        Box::new(TestsApi::instantiate(store, component, linker).unwrap())
    }
}

impl UntypedPluginBinding for TestsApi {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn call_api() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let mut engine = PluginEngine::<Empty>::new(EmptyFactory)?;
    engine.add_capability(
        "tests-api",
        CapabilityWriteRules::SingleWrite,
        TestsApiCapProvider,
    );

    for _ in 0..60 {
        std::thread::sleep(Duration::from_secs_f32(0.016));
        engine.load_modules();
    }

    let mut api = engine.get_single_write_bindings::<TestsApi>("tests-api");
    let mut store = api.store();

    assert!(api.call_get_value(&mut store)? == 0);
    api.call_add_value(&mut store, 42)?;

    assert!(api.call_get_value(&mut store)? == 42);
    api.call_add_value(&mut store, 42)?;

    assert!(api.call_get_value(&mut store)? == 84);
    Ok(())
}
