use crate::{
    config::Config,
    context::ExecutionContext,
    env::PluginEnvironment,
    general::General,
    loader::{FusionPackage, LoaderConfig, PluginLoader},
    manifest::Manifest,
    table::{CapabilityProvider, CapabilityTable, CapabilityWriteRules},
};
use serde::Deserialize;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};
use wasmtime::{Engine, Store};
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
    plugins: HashMap<PluginID, PluginEnvironment<I>>,
    factory: I::Factory,
    failed: Vec<FailedModule>,
    hotswap: HashMap<PluginID, PluginEnvironment<I>>,
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
        log::debug!("[Engine] Initializing plugin engine");
        Self::ensure_directory_exists()?;

        let engine = Engine::default();
        let loader = PluginLoader::new::<I>(loader_config)?;

        Ok(Self {
            engine,
            loader,
            captable: CapabilityTable::default(),
            plugins: HashMap::default(),
            factory,
            failed: vec![],
            hotswap: HashMap::default(),
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
        let environment = self.plugins.get_mut(plugin_id).unwrap();
        let binding_id = TypeId::of::<B>();
        let bindings = environment.bindings_mut();
        let binding = bindings.inner.get(&binding_id).unwrap();
        let binding = binding.as_any().downcast_ref::<B>().unwrap();
        BindingContext {
            store: &mut bindings.store,
            binding,
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
    ) -> Result<(General, PluginID, PluginEnvironment<I>), Box<dyn std::error::Error>> {
        log::warn!("[{}] Preparing module", package.manifest.name());

        let plugin_id = PluginID(package.manifest.id().to_string());
        let mut linker = self.create_linker()?;
        self.captable
            .link(package.manifest.capabilities(), &mut linker, &plugin_id)?;

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

    fn prepare_plugin_direct(
        &mut self,
        package: FusionPackage,
    ) -> Result<General, Box<dyn std::error::Error>> {
        let (general, plugin_id, env) = self.prepare_plugin(package)?;
        self.plugins.insert(plugin_id, env);
        Ok(general)
    }

    fn prepare_plugin_hotswap(
        &mut self,
        package: FusionPackage,
    ) -> Result<General, Box<dyn std::error::Error>> {
        let (general, plugin_id, env) = self.prepare_plugin(package)?;
        self.hotswap.insert(plugin_id, env);
        Ok(general)
    }

    fn call_general_api(&mut self, general: &General, path: &Path, manifest: &Manifest) {
        let (plugin_id, env) = unsafe {
            // SAFETY: The plugin is guaranteed to be valid because it was just added in self.prepare_module.
            self.plugins.iter_mut().last().unwrap_unchecked()
        };

        if let Err(err) = general.call_init(&mut env.bindings_mut().store) {
            let plugin_id = plugin_id.clone();
            let path = path.to_path_buf();
            let manifest = manifest.clone();

            log::error!("[{}] Unable to initialize plugin: {err}", manifest.name(),);
            self.plugins.remove(&plugin_id);
            self.captable
                .remove_observing(manifest.capabilities(), &plugin_id);
            self.failed.push(FailedModule { path, manifest });
        }
    }

    pub fn load_packages(&mut self) {
        let packages = self.loader.get_packages().unwrap();

        for package in packages {
            let path = package.path.clone();
            let manifest = package.manifest.clone();

            if self.plugins.contains_key(package.manifest.id()) {
                log::debug!("Start hotswapping");
                match self.prepare_plugin_hotswap(package) {
                    Ok(api) => {
                        self.call_general_api(&api, &path, &manifest);
                        let plugin = self.hotswap.remove(manifest.id()).unwrap();
                        self.plugins.insert(manifest.id().clone(), plugin);
                    }
                    Err(err) => {
                        log::error!("[Engine] Unable to hotswap plugin: {err}");
                    }
                }
            } else {
                log::debug!("[Engine] Loading module: {}", package.manifest.name());

                match self.prepare_plugin_direct(package) {
                    Ok(api) => {
                        self.call_general_api(&api, &path, &manifest);
                    }
                    Err(err) => {
                        log::error!("[{}] Unable to prepare module: {}", manifest.name(), err);
                        self.failed.push(FailedModule { path, manifest });
                    }
                }
            }
        }
    }

    pub fn restart_plugin(&mut self, plugin_id: impl Into<PluginID>) {
        let plugin_id = plugin_id.into();
        let env = self.plugins.remove(&plugin_id).unwrap();
        log::info!("[Engine] Restart plugin: {}", env.manifest().name());
        self.captable
            .remove_observing(env.manifest().capabilities(), &plugin_id);
        self.loader.load_plugin(env.path_owned()).unwrap();
    }

    pub fn get_plugins(&self) -> Vec<PluginID> {
        self.plugins.keys().cloned().collect()
    }

    pub const fn get_failed_plugins(&self) -> &[FailedModule] {
        self.failed.as_slice()
    }

    pub fn load_package(&mut self, path: PathBuf) {
        self.loader.load_plugin(path).unwrap();
    }
}

pub trait UntypedPluginBinding: 'static {
    fn type_id(&self) -> TypeId;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Default, Debug, Copy, Clone)]
pub enum PluginStatus {
    #[default]
    Running,
    Stopped,
    Traped,
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

pub struct FailedModule {
    path: PathBuf,
    manifest: Manifest,
}

impl FailedModule {
    #[must_use]
    pub const fn path(&self) -> &PathBuf {
        &self.path
    }

    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}
