use std::{
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};

use crate::{
    capabilities::get_imports,
    context::ExecutionContext,
    loader::{ModuleLoader, PackedModule},
    manifest::Manifest,
    stdlib::StandardLibrary,
    table::CapabilityTable,
};
use graphics::graphics::Graphics;
use log::{error, info};
use tokio::sync::Mutex as TokioMutex;
use wasmtime::{Engine, Instance, Memory, Module, Store};

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

pub struct ModuleEngine {
    engine: Engine,
    loader: Arc<TokioMutex<ModuleLoader>>,
    table: CapabilityTable,

    failed: Vec<FailedModule>,
    running: Vec<ModuleWorkspace>,
    stopped: Vec<ModuleWorkspace>,

    //socket: UnixListener,
    graphics: Arc<StdMutex<Graphics>>,
}

impl ModuleEngine {
    const SOCKET_PATH: &str = "nethalym-engine.sock";

    pub fn new(graphics: Arc<StdMutex<Graphics>>) -> Result<Self, Box<dyn std::error::Error>> {
        ensure_directory_exists();

        let _ = std::fs::remove_file(Self::SOCKET_PATH);

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
            failed: Vec::new(),
            running: Vec::new(),
            stopped: Vec::new(),
            //socket,
            graphics,
        })
    }

    fn prepare_module(&mut self, packed: PackedModule) -> Result<(), Box<dyn std::error::Error>> {
        info!("[{}] Preparing module", packed.manifest.name());

        let log_file = dirs::config_dir()
            .unwrap()
            .join("nethalym")
            .join("logs")
            .join(packed.manifest.name());

        let context = ExecutionContext::new(
            self.graphics.clone(),
            packed.config,
            log_file,
            packed.manifest.capabilities(),
        );
        let mut store = Store::new(&self.engine, context);
        let module = Module::from_binary(&self.engine, &packed.module)?;

        let imports = get_imports(module.imports(), &mut store);
        let instance = Instance::new(&mut store, &module, &imports)?;

        let mem = instance
            .get_export(&mut store, "memory")
            .unwrap()
            .into_memory()
            .unwrap();

        mem.grow(&mut store, 2048)?; // 128 MB

        let stdlib = StandardLibrary::new(&instance, &mut store)?;
        let manifest = packed.manifest;
        let path = packed.path;

        self.running.push(ModuleWorkspace {
            path,
            manifest,
            store,
            instance,
            stdlib,
        });

        Ok(())
    }

    pub fn load_modules(&mut self) {
        let modules = if let Ok(mut loader) = self.loader.try_lock() {
            loader.get_raw_modules()
        } else {
            return;
        };

        for module in modules {
            println!("[Engine] Loading module: {}", module.manifest.name());
            let path = module.path.clone();
            let manifest = module.manifest.clone();

            if let Err(err) = self.prepare_module(module) {
                error!("[{}] Unable to prepare module: {}", manifest.name(), err);
                self.failed.push(FailedModule { path, manifest });
                continue;
            }
            let last = self.running.last_mut().unwrap();
            if let Err(err) = last.init() {
                error!("[{}] Unable to initialize module: {}", manifest.name(), err);
                self.failed.push(FailedModule { path, manifest });
            }
        }
    }

    pub fn tick(&mut self) {
        self.load_modules();

        let mut i = 0;
        while i < self.running.len() {
            let module = &mut self.running[i];

            let mut keep = true;

            if let Err(err) = module.tick() {
                error!(
                    "[{}] Failed to tick module: {}",
                    module.manifest.name(),
                    err
                );

                if module.is_support_restore() {
                    match module.get_restore_state() {
                        Err(err) => {
                            error!(
                                "[{}] Failed to create restore state: {}",
                                module.manifest.name(),
                                err
                            );
                            self.failed.push(FailedModule {
                                path: module.path.clone(),
                                manifest: module.manifest.clone(),
                            });
                            keep = false;
                        }
                        Ok(state) => {
                            if state.len() < 4 {
                                error!(
                                    "[{}] Failed to restore module: Invalid state length",
                                    module.manifest.name()
                                );
                                self.failed.push(FailedModule {
                                    path: module.path.clone(),
                                    manifest: module.manifest.clone(),
                                });
                                keep = false;
                            } else if let Err(err) = module.restore(state) {
                                error!(
                                    "[{}] Failed to restore module: {}",
                                    module.manifest.name(),
                                    err
                                );
                                self.failed.push(FailedModule {
                                    path: module.path.clone(),
                                    manifest: module.manifest.clone(),
                                });
                                keep = false;
                            } else {
                                info!("[{}] Module restored successfully", module.manifest.name());
                            }
                        }
                    }
                } else {
                    error!(
                        "[{}] Module does not support restore",
                        module.manifest.name()
                    );

                    self.failed.push(FailedModule {
                        path: module.path.clone(),
                        manifest: module.manifest.clone(),
                    });

                    keep = false;
                }
            }

            if keep {
                i += 1;
            } else {
                self.running.remove(i); // удаляем из вектора
            }
        }
    }

    /*
    fn prepare_module_list(&self, filter: ModuleListFilter) -> Vec<String> {
        match filter {
            ModuleListFilter::All => self
                .running
                .iter()
                .map(|m| m.manifest.name().to_string())
                .chain(self.stopped.iter().map(|m| m.manifest.name().to_string()))
                .chain(self.failed.iter().map(|m| m.manifest.name().to_string()))
                .collect::<Vec<_>>(),

            ModuleListFilter::Failed => self
                .failed
                .iter()
                .map(|m| m.manifest.name().to_string())
                .collect::<Vec<_>>(),

            ModuleListFilter::Running => self
                .running
                .iter()
                .map(|m| m.manifest.name().to_string())
                .collect::<Vec<_>>(),

            ModuleListFilter::Stopped => self
                .stopped
                .iter()
                .map(|m| m.manifest.name().to_string())
                .collect::<Vec<_>>(),
        }
    }

    async fn handle_socket_server(&mut self) -> std::io::Result<()> {
        //let socket = self.socket.try_clone().unwrap();
        //let Ok((mut stream, _)) = socket.accept() else {
        //    return Ok(());
        //};

        //let mut buffer = [0; 1024];
        //match stream.read(&mut buffer) {
        //    Ok(size) if size > 0 => {
        //        let (command, _): (SocketCommand, _) =
        //            bincode::decode_from_slice(&buffer[0..size], bincode::config::standard())
        //                .unwrap();

        //        self.execute_socket_command(&mut stream, command);
        //    }
        //    Ok(_) => info!("Client disconnected"),
        //    Err(e) => eprintln!("Read error: {}", e),
        //}

        Ok(())
    }

    async fn execute_socket_command(&mut self, stream: &mut UnixStream, command: SocketCommand) {
        match command {
            SocketCommand::Modules { filter } => {
                let result = SocketCommandResult::Modules {
                    list: self.prepare_module_list(filter.unwrap_or_default()),
                };
                let bytes = bincode::encode_to_vec(result, bincode::config::standard()).unwrap();
                stream.write_all(&bytes).await.unwrap();
            }
            SocketCommand::ReloadModule { id } => {
                self.restart_module(id);
                let bytes =
                    bincode::encode_to_vec(SocketCommandResult::Done, bincode::config::standard())
                        .unwrap();
                stream.write_all(&bytes).await.unwrap();
            }
        }
    }


    pub fn restart_module(&mut self, index: usize) {
        let module = self.failed.remove(index);
        ModuleLoader::create_packed_module(module.path).unwrap();
        //TODO: Implement module restart logic
    }

    */
}

pub struct ModuleWorkspace {
    path: String,
    manifest: Manifest,
    store: Store<ExecutionContext>,
    instance: Instance,
    stdlib: StandardLibrary,
}

impl ModuleWorkspace {
    pub const fn new(
        path: String,
        manifest: Manifest,
        store: Store<ExecutionContext>,
        instance: Instance,
        stdlib: StandardLibrary,
    ) -> Self {
        ModuleWorkspace {
            path,
            manifest,
            store,
            instance,
            stdlib,
        }
    }

    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("[{}] Initializing module", self.manifest.name());
        self.stdlib.init(&mut self.store)?;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.stdlib.tick(&mut self.store)?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.stdlib.stop(&mut self.store)?;
        Ok(())
    }

    pub fn is_support_restore(&self) -> bool {
        self.stdlib.is_support_restore()
    }

    pub fn get_restore_state(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        debug_assert!(self.is_support_restore());
        let memory = self.get_memory();
        self.stdlib.get_restore_state(memory, &mut self.store)
    }

    pub fn restore(&mut self, state: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let memory = self.get_memory();
        self.stdlib.restore(&mut self.store, memory, state)
    }

    fn get_memory(&mut self) -> Memory {
        self.instance
            .get_export(&mut self.store, "memory")
            .unwrap()
            .into_memory()
            .unwrap()
    }
}

pub struct FailedModule {
    path: String,
    manifest: Manifest,
}
