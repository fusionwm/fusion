use crate::{
    context::ExecutionContext,
    general::General,
    loader::{ModuleLoader, PackedModule},
    manifest::Manifest,
    table::{CapabilityProvider, CapabilityTable, CapabilityWriteRules},
};
use slotmap::{SlotMap, new_key_type};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex as TokioMutex;
use wasmtime::{Engine, Store};
use wasmtime::{
    StoreContextMut,
    component::{Component, Linker},
};

fn ensure_directory_exists() {
    let root = dirs::config_dir().unwrap().join("nethalym");
    if !root.exists() {
        std::fs::create_dir(&root).unwrap();
    }

    let modules = root.join("modules");
    if !modules.exists() {
        std::fs::create_dir(&modules).unwrap();
    }

    let logs = root.join("logs");
    if !logs.exists() {
        std::fs::create_dir(&logs).unwrap();
    }
}

pub trait InnerContextFactory<I: InnerContext> {
    fn generate(&self, capabilities: &[String]) -> I;
}

pub trait InnerContext: Send + Sync + Sized + 'static {
    type Factory: InnerContextFactory<Self>;
}

new_key_type! { pub(crate) struct PluginID; }

pub struct ModuleEngine<I: InnerContext> {
    engine: Engine,
    loader: Arc<TokioMutex<ModuleLoader>>,
    table: CapabilityTable<I>,
    plugins: SlotMap<PluginID, PluginEnvironment<I>>,
    factory: I::Factory,
    failed: Vec<FailedModule>,
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

impl<I: InnerContext> ModuleEngine<I> {
    pub fn new(factory: I::Factory) -> Result<Self, Box<dyn std::error::Error>> {
        ensure_directory_exists();

        let engine = Engine::default();
        let loader = Arc::new(TokioMutex::new(ModuleLoader::new()?));
        let loader_clone = loader.clone();
        tokio::task::spawn(async move {
            loop {
                if let Ok(mut loader) = loader_clone.try_lock() {
                    loader.handle_events();
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        Ok(Self {
            engine,
            loader,
            table: CapabilityTable::default(),
            plugins: SlotMap::default(),
            factory,
            failed: vec![],
        })
    }

    pub fn add_capability(
        &mut self,
        identifier: impl Into<String>,
        kind: CapabilityWriteRules,
        provider: impl CapabilityProvider<Inner = I>,
    ) {
        self.table
            .register_capability(identifier.into(), kind, provider);
    }

    pub fn get_single_write_bindings<B: UntypedPluginBinding>(
        &mut self,
        capability: &str,
    ) -> BindingContext<'_, I, B> {
        let capability = self.table.get_capability_by_name(capability);
        let plugin_id = capability.writers().iter().next().unwrap();
        let environment = self.plugins.get_mut(*plugin_id).unwrap();
        let binding_id = TypeId::of::<B>();
        let binding = environment.bindings.inner.get(&binding_id).unwrap();
        let binding = binding.as_any().downcast_ref::<B>().unwrap();
        BindingContext {
            store: &mut environment.bindings.store,
            binding,
        }
    }

    fn prepare_module(
        &mut self,
        packed: PackedModule,
    ) -> Result<General, Box<dyn std::error::Error>> {
        log::warn!("[{}] Preparing module", packed.manifest.name());

        let log_file = dirs::config_dir()
            .unwrap()
            .join("nethalym")
            .join("logs")
            .join(packed.manifest.name());

        let mut linker = Linker::<ExecutionContext<I>>::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        let component = Component::from_binary(&self.engine, &packed.module)?;
        General::add_to_linker::<_, ExecutionContext<I>>(&mut linker, |store| store)?;
        self.table.link(packed.manifest.capabilities(), &mut linker);

        let inner_context = self.factory.generate(packed.manifest.capabilities());
        let context = ExecutionContext::new(packed.config, log_file, inner_context);

        let store = Store::new(&self.engine, context);

        let plugin_id = self.plugins.insert(PluginEnvironment {
            path: packed.path,
            manifest: packed.manifest,
            component,
            status: PluginStatus::Running,
            bindings: Bindings::new(store),
        });

        // SAFETY: The plugin ID is guaranteed to be valid because it was just inserted.
        let (bindings, capabilities, component) = unsafe {
            let workspace = self.plugins.get_mut(plugin_id).unwrap_unchecked();
            (
                &mut workspace.bindings,
                workspace.manifest.capabilities(),
                &workspace.component,
            )
        };

        let _ = linker.define_unknown_imports_as_traps(component);
        let general = General::instantiate(&mut bindings.store, component, &linker)?;

        self.table.observe_if_needed(capabilities, plugin_id);
        self.table
            .create_bindings(capabilities, bindings, component, &mut linker);

        Ok(general)
    }

    pub fn load_modules(&mut self) {
        let modules = if let Ok(mut loader) = self.loader.try_lock() {
            loader.get_raw_modules()
        } else {
            return;
        };

        println!("Raw modules: {}", modules.len());

        for module in modules {
            log::debug!("[Engine] Loading module: {}", module.manifest.name());
            let path = module.path.clone();
            let manifest = module.manifest.clone();

            match self.prepare_module(module) {
                Ok(general) => {
                    let (plugin_id, env) = unsafe {
                        // SAFETY: The plugin is guaranteed to be valid because it was just added in self.prepare_module.
                        self.plugins.iter_mut().last().unwrap_unchecked()
                    };

                    if let Err(err) = general.call_init(&mut env.bindings.store) {
                        log::error!("[{}] Unable to initialize module: {}", manifest.name(), err);
                        self.plugins.remove(plugin_id);
                        self.table
                            .remove_observing(manifest.capabilities(), plugin_id);
                        self.failed.push(FailedModule { path, manifest });
                    }
                }
                Err(err) => {
                    log::error!("[{}] Unable to prepare module: {}", manifest.name(), err);
                    self.failed.push(FailedModule { path, manifest });
                }
            }
        }
    }

    pub fn handle_modules(&mut self) {}

    pub fn tick(&mut self) {
        self.load_modules();
    }

    /*

    pub fn restart_module(&mut self, index: usize) {
        let module = self.failed.remove(index);
        ModuleLoader::create_packed_module(module.path).unwrap();
        //TODO: Implement module restart logic
    }

    */
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

#[allow(dead_code)]
pub struct PluginEnvironment<I: InnerContext> {
    path: String,
    manifest: Manifest,
    component: Component,
    status: PluginStatus,
    bindings: Bindings<I>,
}

impl<I: InnerContext> PluginEnvironment<I> {
    #[must_use]
    pub const fn path(&self) -> &str {
        self.path.as_str()
    }

    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    #[must_use]
    pub const fn status(&self) -> PluginStatus {
        self.status
    }
}

pub struct FailedModule {
    path: String,
    manifest: Manifest,
}
