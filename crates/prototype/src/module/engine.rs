use crate::{
    ModuleListFilter, SocketCommand, SocketCommandResult,
    capabilities::get_imports,
    module::{
        context::ExecutionContext,
        loader::{ModuleLoader, PackedModule},
        manifest::Manifest,
        stdlib::StandardLibrary,
        table::CapabilityTable,
    },
};
use log::{error, info};
use tokio::{
    io::AsyncWriteExt,
    net::{UnixListener, UnixStream},
};
use wasmtime::{Engine, Instance, Memory, Module, Store};

pub enum EngineEvent {
    Loader,
    Engine,
}

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
    loader: ModuleLoader,
    table: CapabilityTable,

    failed: Vec<FailedModule>,
    running: Vec<ModuleWorkspace>,
    stopped: Vec<ModuleWorkspace>,

    socket: UnixListener,
}

impl ModuleEngine {
    pub fn new(loader: ModuleLoader) -> Self {
        ensure_directory_exists();

        const SOCKET_PATH: &str = "nethalym-engine.sock";
        let _ = std::fs::remove_file(SOCKET_PATH);

        let mut engine_config = wasmtime::Config::new();
        engine_config.async_support(true);
        let engine = Engine::new(&engine_config).unwrap();

        let socket = UnixListener::bind(SOCKET_PATH).unwrap();

        Self {
            engine,
            loader,
            table: CapabilityTable::default(),
            failed: Vec::new(),
            running: Vec::new(),
            stopped: Vec::new(),
            socket,
        }
    }

    async fn prepare_module(&mut self, packed: PackedModule) -> Result<(), crate::Error> {
        info!("[{}] Preparing module", packed.manifest.name());

        let log_file = dirs::config_dir()
            .unwrap()
            .join("nethalym/logs")
            .join(packed.manifest.name());
        println!("Log file path: {}", log_file.display());
        let context =
            ExecutionContext::new(packed.config, log_file, packed.manifest.capabilities());
        let mut store = Store::new(&self.engine, context);
        let module = Module::from_binary(&self.engine, &packed.module)?;

        let imports = get_imports(module.imports(), &mut store);
        let instance = Instance::new_async(&mut store, &module, &imports).await?;

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

    pub async fn tick(&mut self) -> Result<(), crate::Error> {
        let loaded = self.loader.get_raw_modules();
        for module in loaded {
            let path = module.path.clone();
            let manifest = module.manifest.clone();

            if let Err(err) = self.prepare_module(module).await {
                error!("[{}] Unable to prepare module: {}", manifest.name(), err);
                self.failed.push(FailedModule { path, manifest });
                continue;
            }
            let last = self.running.last_mut().unwrap();
            if let Err(err) = last.init().await {
                error!("[{}] Unable to initialize module: {}", manifest.name(), err);
                self.failed.push(FailedModule { path, manifest });
            }
        }

        let mut i = 0;
        while i < self.running.len() {
            let module = &mut self.running[i];

            let mut keep = true;

            if let Err(err) = module.tick().await {
                error!(
                    "[{}] Failed to tick module: {}",
                    module.manifest.name(),
                    err
                );

                if module.is_support_restore() {
                    match module.get_restore_state().await {
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
                            } else if let Err(err) = module.restore(state).await {
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
        Ok(())
    }

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

    pub fn poll_events(&mut self) {}

    pub async fn handle_events(&mut self) {
        //self.handle_socket_server().await.unwrap();
        self.loader.handle_events().await;
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
}

pub struct ModuleWorkspace {
    path: String,
    manifest: Manifest,
    store: Store<ExecutionContext>,
    instance: Instance,
    stdlib: StandardLibrary,
}

impl ModuleWorkspace {
    pub async fn init(&mut self) -> Result<(), crate::Error> {
        info!("[{}] Initializing module", self.manifest.name());
        self.stdlib.init(&mut self.store).await?;
        Ok(())
    }

    pub async fn tick(&mut self) -> Result<(), crate::Error> {
        self.stdlib.tick(&mut self.store).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), crate::Error> {
        self.stdlib.stop(&mut self.store).await?;
        Ok(())
    }

    pub fn is_support_restore(&self) -> bool {
        self.stdlib.is_support_restore()
    }

    pub async fn get_restore_state(&mut self) -> Result<Vec<u8>, crate::Error> {
        debug_assert!(self.is_support_restore());
        let memory = self.get_memory();
        self.stdlib.get_restore_state(memory, &mut self.store).await
    }

    pub async fn restore(&mut self, state: Vec<u8>) -> Result<(), crate::Error> {
        let memory = self.get_memory();
        self.stdlib.restore(&mut self.store, memory, state).await
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
