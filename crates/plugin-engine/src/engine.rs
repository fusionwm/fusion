use crate::{
    config::Config,
    context::ExecutionContext,
    env::PluginEnvironment,
    general::General,
    loader::{FusionPackage, LoaderConfig, PluginLoader},
    manifest::Manifest,
    table::{CapabilityProvider, CapabilityTable, CapabilityWriteRules},
};
use derive_more::Display;
use serde::Deserialize;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};
use wasmtime::{Engine, InstanceAllocationStrategy, Store};
use wasmtime::{
    StoreContextMut,
    component::{Component, Linker},
};

pub trait InnerContextFactory<I: InnerContext> {
    fn generate(&self, capabilities: &[String]) -> I;
}

pub trait InnerContext: Send + Sync + Sized + 'static {
    type Factory: InnerContextFactory<Self>;
    fn config_path() -> PathBuf;
    fn logs_path() -> PathBuf;
    fn plugins_path() -> PathBuf;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct PluginID(String);

impl Display for PluginID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PluginID {
    fn from(value: String) -> Self {
        PluginID(value)
    }
}

impl From<&str> for PluginID {
    fn from(value: &str) -> Self {
        PluginID(value.to_string())
    }
}

pub struct PluginEngine<I: InnerContext> {
    engine: Engine,
    loader: PluginLoader,
    captable: CapabilityTable<I>,
    plugins: HashMap<PluginID, Plugin<I>>,
    factory: I::Factory,
}

pub struct BindingContext<'environment, I: InnerContext, B: UntypedPluginBinding> {
    store: &'environment mut Store<ExecutionContext<I>>,
    pub binding: &'environment B,
}

impl<I: InnerContext, B: UntypedPluginBinding> BindingContext<'_, I, B> {
    pub fn store(&mut self) -> UnsafeStore<I> {
        UnsafeStore {
            store: core::ptr::from_mut(self.store),
        }
    }
}

pub struct UnsafeStore<I: InnerContext> {
    store: *mut Store<ExecutionContext<I>>,
}

impl<I: InnerContext> wasmtime::AsContext for UnsafeStore<I> {
    type Data = ExecutionContext<I>;

    fn as_context(&self) -> wasmtime::StoreContext<'_, Self::Data> {
        unsafe {
            let store = &mut *self.store;
            std::mem::transmute(store.as_context())
        }
    }
}

impl<I: InnerContext> wasmtime::AsContextMut for UnsafeStore<I> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, Self::Data> {
        unsafe {
            let store = &mut *self.store;
            std::mem::transmute(store.as_context_mut())
        }
    }
}

impl<I: InnerContext, B: UntypedPluginBinding> core::ops::Deref for BindingContext<'_, I, B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        self.binding
    }
}

impl<I: InnerContext> PluginEngine<I> {
    fn ensure_directory_exists() -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(I::config_path())?;
        std::fs::create_dir_all(I::plugins_path())?;
        std::fs::create_dir_all(I::logs_path())?;
        Ok(())
    }

    pub fn new(
        factory: I::Factory,
        loader_config: LoaderConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::debug!("[Engine] Initializing...");
        Self::ensure_directory_exists()?;

        let mut config = wasmtime::Config::new();
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        config.compiler_inlining(true);
        config.wasm_simd(true);
        config.allocation_strategy(InstanceAllocationStrategy::pooling());
        let engine = Engine::new(&config)?;
        let loader = PluginLoader::new::<I>(loader_config)?;

        Ok(Self {
            engine,
            loader,
            captable: CapabilityTable::default(),
            plugins: HashMap::default(),
            factory,
        })
    }

    pub fn add_capability(
        &mut self,
        identifier: impl Into<String>,
        kind: CapabilityWriteRules,
        provider: impl CapabilityProvider<Inner = I>,
    ) {
        self.captable
            .register_capability(identifier.into(), kind, provider);
    }

    pub fn get_single_write_bindings<B: UntypedPluginBinding>(
        &mut self,
        capability: &str,
    ) -> BindingContext<'_, I, B> {
        let capability = self.captable.get_capability_by_name(capability);
        let plugin_id = capability.writers().iter().next().unwrap();
        match self.plugins.get_mut(plugin_id).unwrap() {
            Plugin::Running(env) => {
                let binding_id = TypeId::of::<B>();
                let bindings = env.bindings_mut();
                let binding = bindings.inner.get(&binding_id).unwrap();
                let binding = binding.as_any().downcast_ref::<B>().unwrap();
                BindingContext {
                    store: &mut bindings.store,
                    binding,
                }
            }
            Plugin::Failed(_) => {
                panic!("Failed plugin cannot provide bindings");
            }
        }
    }

    fn create_context(&self, manifest: &Manifest, config: Config) -> ExecutionContext<I> {
        let log_file = I::logs_path().join(manifest.name());
        let inner_context = self.factory.generate(manifest.capabilities());
        ExecutionContext::new(config, log_file, inner_context)
    }

    fn create_linker(&self) -> Result<Linker<ExecutionContext<I>>, Box<dyn std::error::Error>> {
        let mut linker = Linker::<ExecutionContext<I>>::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        Ok(linker)
    }

    fn prepare_plugin(
        &mut self,
        package: FusionPackage,
        silent_link: bool,
    ) -> Result<(General, PluginID, PluginEnvironment<I>), Box<dyn std::error::Error>> {
        log::warn!("[{}] Preparing plugin", package.manifest.name());

        let plugin_id = PluginID(package.manifest.id().to_string());
        let mut linker = self.create_linker()?;
        self.captable.link(
            package.manifest.capabilities(),
            &mut linker,
            &plugin_id,
            silent_link,
        )?;

        let context = self.create_context(&package.manifest, package.config);
        let store = Store::new(&self.engine, context);
        let component = Component::from_binary(&self.engine, &package.module)?;
        let _ = linker.define_unknown_imports_as_traps(&component);

        let mut env = PluginEnvironment::new(
            package.path,
            package.manifest,
            component,
            Bindings::new(store),
        );

        env.create_bindings(&mut linker, &mut self.captable);
        let general = env.instantiate_general_api(&linker)?;

        Ok((general, plugin_id, env))
    }

    fn call_general_api(
        &mut self,
        api: &General,
        plugin_id: PluginID,
        mut env: PluginEnvironment<I>,
        path: &Path,
        manifest: &Manifest,
    ) {
        let plugin = if let Err(err) = api.call_init(&mut env.bindings_mut().store) {
            let plugin_id = plugin_id.clone();
            let path = path.to_path_buf();
            let manifest = manifest.clone();

            log::error!("[{}] Unable to initialize plugin: {err}", manifest.name(),);
            self.captable
                .remove_observing(manifest.capabilities(), &plugin_id);
            Plugin::Failed(FailedPlugin { path, manifest })
        } else {
            Plugin::Running(env)
        };

        self.plugins.insert(plugin_id, plugin);
    }

    pub fn load_packages(&mut self) {
        let Ok(packages) = self.loader.get_packages() else {
            log::error!("[Engine] Failed to get packages from loader");
            return;
        };

        for package in packages {
            let id = package.manifest.id().clone();
            let name = package.manifest.name().to_string();
            let silent_link = self.plugins.contains_key(&id);

            log::debug!(
                "[Engine] {} plugin: {}",
                if silent_link {
                    "Hotswapping"
                } else {
                    "Loading"
                },
                name
            );

            match self.prepare_plugin(package.clone(), silent_link) {
                Ok((api, plugin_id, env)) => {
                    self.call_general_api(&api, plugin_id, env, &package.path, &package.manifest);
                }
                Err(err) => {
                    log::error!(
                        "[{}] Unable to {} plugin: {}",
                        name,
                        if silent_link { "hotswap" } else { "prepare" },
                        err
                    );
                    if !silent_link {
                        self.plugins.insert(
                            package.manifest.id().clone(),
                            Plugin::Failed(FailedPlugin {
                                path: package.path,
                                manifest: package.manifest,
                            }),
                        );
                    }
                }
            }
        }
    }

    pub fn restart_plugin(&mut self, plugin_id: impl Into<PluginID>) -> Result<(), Error> {
        let plugin_id = plugin_id.into();
        if let Some(plugin) = self.plugins.remove(&plugin_id) {
            log::info!("[Engine] Restart plugin: {}", plugin.manifest().name());
            self.captable
                .remove_observing(plugin.manifest().capabilities(), &plugin_id);
            self.loader.load_plugin(plugin.path()).unwrap();
            Ok(())
        } else {
            log::error!("[Engine] Plugin with ID '{plugin_id}' not found");
            Err(Error::PluginNotFound(plugin_id.to_string()))
        }
    }

    pub fn get_plugin_list(&self) -> Vec<PluginID> {
        self.plugins.keys().cloned().collect()
    }

    pub fn get_failed_plugins(&self) -> Vec<&FailedPlugin> {
        self.plugins
            .iter()
            .filter(|(_, v)| matches!(v, Plugin::Failed(_)))
            .map(|(_, v)| match v {
                Plugin::Failed(failed) => failed,
                Plugin::Running(_) => unreachable!(),
            })
            .collect()
    }

    pub fn load_package(&mut self, path: PathBuf) {
        self.loader.load_plugin(path).unwrap();
    }

    pub fn get_plugin_env_by_id(&self, plugin_id: &PluginID) -> Option<&Plugin<I>> {
        self.plugins.get(plugin_id)
    }
}

pub trait UntypedPluginBinding: 'static {
    fn type_id(&self) -> TypeId;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(crate) struct Bindings<I: InnerContext> {
    store: Store<ExecutionContext<I>>,
    inner: HashMap<TypeId, Box<dyn UntypedPluginBinding>>,
}

impl<I: InnerContext> Bindings<I> {
    pub fn new(store: Store<ExecutionContext<I>>) -> Self {
        Self {
            store,
            inner: HashMap::new(),
        }
    }

    pub fn add(&mut self, bindings: Box<dyn UntypedPluginBinding>) {
        self.inner.insert((*bindings).type_id(), bindings);
    }

    pub const fn store_mut(&mut self) -> &mut Store<ExecutionContext<I>> {
        &mut self.store
    }
}

pub struct FailedPlugin {
    path: PathBuf,
    manifest: Manifest,
}

impl FailedPlugin {
    #[must_use]
    pub const fn path(&self) -> &PathBuf {
        &self.path
    }

    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub enum PluginStatus {
    #[default]
    Running,
    Failed,
}

pub enum Plugin<I: InnerContext> {
    Running(PluginEnvironment<I>),
    Failed(FailedPlugin),
}

impl<I: InnerContext> Plugin<I> {
    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        match self {
            Plugin::Running(env) => env.manifest(),
            Plugin::Failed(failed) => failed.manifest(),
        }
    }

    #[must_use]
    pub fn path(&self) -> PathBuf {
        match self {
            Plugin::Running(env) => env.path().clone(),
            Plugin::Failed(failed) => failed.path().clone(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> &PluginID {
        match self {
            Plugin::Running(env) => env.manifest().id(),
            Plugin::Failed(failed) => failed.manifest().id(),
        }
    }

    #[must_use]
    pub const fn status(&self) -> PluginStatus {
        match self {
            Plugin::Running(_) => PluginStatus::Running,
            Plugin::Failed(_) => PluginStatus::Failed,
        }
    }
}

pub enum Error {
    PluginNotFound(String),
}
