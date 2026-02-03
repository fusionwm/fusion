#![allow(clippy::non_std_lazy_statics)]

use lazy_static::lazy_static;
use std::{
    any::{Any, TypeId},
    path::{Path, PathBuf},
    time::Duration,
};
use tempfile::TempDir;

use plugin_engine::{
    InnerContext, InnerContextFactory, PluginEngine, UntypedPluginBinding,
    context::ExecutionContext,
    impl_untyped_plugin_binding,
    table::{CapabilityProvider, CapabilityWriteRules},
    wasm::{Component, Linker, Store},
};

lazy_static! {
    static ref PLUGINS_PATH: TempDir = tempfile::tempdir().unwrap();
    static ref LOGS_PATH: TempDir = tempfile::tempdir().unwrap();
    static ref CONFIG_PATH: TempDir = tempfile::tempdir().unwrap();
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

fn execute_cargo_fusion(working_dir: &Path) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("cargo-fusion");
    cmd.arg("build")
        .arg("-o")
        .arg(PLUGINS_PATH.path())
        .current_dir(working_dir)
        .status()?;

    Ok(())
}

fn build_plugins() -> anyhow::Result<()> {
    let plugins = std::env::current_dir()?.join("tests").join("tests_plugins");
    let dir = std::fs::read_dir(plugins)?;
    for entry in dir {
        let entry = entry?;
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        execute_cargo_fusion(&entry.path())?;
    }

    Ok(())
}

fn wait_one_second(engine: &mut PluginEngine<Empty>) {
    const FRAME_COUNT: u64 = 60;
    let frame_time = Duration::from_secs(1 / FRAME_COUNT);
    for _ in 0..FRAME_COUNT {
        std::thread::sleep(frame_time);
        engine.load_modules();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn call_api() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    build_plugins()?;
    let mut engine = PluginEngine::<Empty>::new(EmptyFactory)?;
    engine.add_capability(
        "tests-api",
        CapabilityWriteRules::SingleWrite,
        TestsApiCapProvider,
    );

    wait_one_second(&mut engine);

    let mut api = engine.get_single_write_bindings::<TestsApi>("tests-api");
    let mut store = api.store();

    assert!(api.call_get_value(&mut store)? == 0);
    api.call_add_value(&mut store, 42)?;

    assert!(api.call_get_value(&mut store)? == 42);
    api.call_add_value(&mut store, 42)?;

    assert!(api.call_get_value(&mut store)? == 84);
    Ok(())
}
