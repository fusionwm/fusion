use crate::{
    config::Config,
    context::ExecutionContext,
    env::PluginEnvironment,
    general::General,
    loader::{ModuleLoader, PackedModule},
    manifest::Manifest,
    table::{CapabilityProvider, CapabilityTable, CapabilityWriteRules},
};
use slotmap::{SlotMap, new_key_type};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex as TokioMutex;
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

new_key_type! { pub(crate) struct PluginID; }

pub struct ModuleEngine<I: InnerContext> {
    engine: Engine,
    loader: Arc<TokioMutex<ModuleLoader>>,
    captable: CapabilityTable<I>,
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
    fn ensure_directory_exists() -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(I::config_path())?;
        std::fs::create_dir_all(I::plugins_path())?;
        std::fs::create_dir_all(I::logs_path())?;
        Ok(())
    }

    pub fn new(factory: I::Factory) -> Result<Self, Box<dyn std::error::Error>> {
        Self::ensure_directory_exists()?;

        let engine = Engine::default();
        let loader = Arc::new(TokioMutex::new(ModuleLoader::new::<I>()?));
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
            captable: CapabilityTable::default(),
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
        self.captable
            .register_capability(identifier.into(), kind, provider);
    }

    pub fn get_single_write_bindings<B: UntypedPluginBinding>(
        &mut self,
        capability: &str,
    ) -> BindingContext<'_, I, B> {
        let capability = self.captable.get_capability_by_name(capability);
        let plugin_id = capability.writers().iter().next().unwrap();
        let environment = self.plugins.get_mut(*plugin_id).unwrap();
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

    fn prepare_module(
        &mut self,
        packed: PackedModule,
    ) -> Result<General, Box<dyn std::error::Error>> {
        log::warn!("[{}] Preparing module", packed.manifest.name());

        let mut linker = self.create_linker()?;
        self.captable
            .link(packed.manifest.capabilities(), &mut linker)?;

        let context = self.create_context(&packed.manifest, packed.config);
        let store = Store::new(&self.engine, context);
        let component = Component::from_binary(&self.engine, &packed.module)?;
        let _ = linker.define_unknown_imports_as_traps(&component);

        let plugin_id = self.plugins.insert(PluginEnvironment::new(
            packed.path,
            packed.manifest,
            component,
            Bindings::new(store),
        ));

        // SAFETY: The plugin ID is guaranteed to be valid because it was just inserted.
        let env = unsafe { self.plugins.get_mut(plugin_id).unwrap_unchecked() };
        env.create_bindings(&mut linker, &mut self.captable, plugin_id);

        env.instantiate_general_api(&linker)
    }

    pub fn load_modules(&mut self) {
        let modules = if let Ok(mut loader) = self.loader.try_lock() {
            loader.get_raw_modules()
        } else {
            return;
        };

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

                    if let Err(err) = general.call_init(&mut env.bindings_mut().store) {
                        log::error!("[{}] Unable to initialize module: {}", manifest.name(), err);
                        self.plugins.remove(plugin_id);
                        self.captable
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

    pub fn has_unloaded_modules(&self) -> bool {
        if let Ok(loader) = self.loader.try_lock() {
            loader.has_packed()
        } else {
            false
        }
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

pub struct FailedModule {
    path: String,
    manifest: Manifest,
}
