use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::Duration,
};

use notify::{
    RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
};
use tokio::sync::Mutex;
use zip::ZipArchive;

use crate::{config::Config, engine::InnerContext, manifest::Manifest};

enum Request {
    GetPlugins,
    LoadPlugin(PathBuf),
}

enum Answer {
    GetPlugins(Vec<PackedModule>),
}

async fn run_loader_loop<I: InnerContext>(loader: Arc<Mutex<InnerPluginLoader>>) {
    {
        let mut loader = loader.lock().await;
        if let Err(e) = loader.preload_plugins(I::plugins_path()).await {
            log::error!("[Loader] Preload failed: {e:?}");
            return;
        }
    }

    let mut interval = tokio::time::interval(Duration::from_millis(10));
    loop {
        interval.tick().await; // Более точный аналог sleep

        if let Ok(mut loader) = loader.try_lock() {
            loader.handle_events();
        }
    }
}

type TokioSender<T> = tokio::sync::mpsc::Sender<T>;
type TokioReceiver<T> = tokio::sync::mpsc::Receiver<T>;

pub(crate) struct PluginLoader {
    loader: Arc<Mutex<InnerPluginLoader>>,
    request_sender: Sender<Request>,
    answer_receiver: Receiver<Answer>,
}

impl PluginLoader {
    pub fn new<I: InnerContext>() -> Result<Self, Box<dyn std::error::Error>> {
        let (request_sender, request_receiver) = std::sync::mpsc::channel();
        let (answer_sender, answer_receiver) = std::sync::mpsc::channel();
        let loader = InnerPluginLoader::new(request_receiver, answer_sender, I::plugins_path())?;
        let loader = Arc::new(Mutex::new(loader));
        tokio::task::spawn(run_loader_loop::<I>(loader.clone()));
        Ok(Self {
            loader,
            request_sender,
            answer_receiver,
        })
    }

    pub fn get_packed_plugins(&mut self) -> Result<Vec<PackedModule>, Box<dyn std::error::Error>> {
        let mut packed = Vec::new();
        while let Ok(answer) = self.answer_receiver.try_recv() {
            match answer {
                Answer::GetPlugins(plugins) => packed.extend(plugins),
            }
        }

        self.request_sender.send(Request::GetPlugins)?;

        Ok(packed)
    }

    pub fn load_plugin(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.request_sender.send(Request::LoadPlugin(path))?;
        Ok(())
    }
}

struct InnerPluginLoader {
    request_receiver: Receiver<Request>,
    answer_sender: Sender<Answer>,

    file_receiver: TokioReceiver<notify::Result<notify::Event>>,
    loaded: HashMap<PathBuf, PackedModule>,
    renamed: Option<PackedModule>,
}

impl InnerPluginLoader {
    pub fn has_packed(&self) -> bool {
        !self.loaded.is_empty()
    }

    pub fn new(
        request_receiver: Receiver<Request>,
        answer_sender: Sender<Answer>,
        plugins_path: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (file_tx, file_receiver) = tokio::sync::mpsc::channel(32);

        tokio::task::spawn_blocking(move || {
            let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
            let mut watcher = notify::recommended_watcher(tx).unwrap(); //TODO Error
            watcher
                .watch(plugins_path.as_path(), RecursiveMode::NonRecursive)
                .unwrap(); //TODO Error

            for event in rx {
                let _ = file_tx.blocking_send(event);
            }
        });

        Ok(Self {
            request_receiver,
            answer_sender,
            file_receiver,
            loaded: HashMap::new(),
            renamed: None,
        })
    }

    fn load_plugin(&mut self, path: PathBuf) {
        match Self::create_packed_module(path.clone()) {
            Ok(module) => {
                log::info!("[Loader] Load module: {}", module.manifest.name());
                if let Some(cap) = module.manifest.custom_capabilities() {
                    log::info!(
                        "[{}] Available capabilities: {:?}",
                        module.manifest.name(),
                        cap
                    );
                }
                self.loaded.insert(path, module);
            }
            Err(error) => log::error!("[Loader] {error}"),
        }
    }

    async fn preload_plugins(
        &mut self,
        plugins_path: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = tokio::fs::read_dir(&plugins_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "lym") {
                self.load_plugin(path);
            }
        }

        Ok(())
    }

    fn handle_events(&mut self) {
        while let Ok(request) = self.request_receiver.try_recv() {
            match request {
                Request::GetPlugins => {
                    let answer = Answer::GetPlugins(self.get_packed_plugins());
                    if let Err(err) = self.answer_sender.send(answer) {
                        log::error!("[Loader] {err}");
                    }
                }
                Request::LoadPlugin(path) => self.load_plugin(path),
            }
        }

        while let Ok(event) = self.file_receiver.try_recv() {
            let mut event = match event {
                Ok(events) => events,
                Err(error) => {
                    log::error!("[Loader] {error}");
                    return;
                }
            };

            match event.kind {
                notify::EventKind::Create(_) => {
                    for path in event.paths {
                        let metadata = std::fs::metadata(&path)
                            .map_err(|error| {
                                log::error!("[Loader] {error}");
                            })
                            .ok()
                            .unwrap();

                        if !metadata.is_file() {
                            return;
                        }

                        self.load_plugin(path);
                    }
                }
                notify::EventKind::Remove(_) => {
                    for path in event.paths {
                        if let Some(module) = self.loaded.remove(&path) {
                            log::info!("[Loader] Unload module: {}", module.manifest.name());
                        }
                    }
                }
                notify::EventKind::Modify(modify_kind) => {
                    let path = event.paths.remove(0);
                    //TODO Fix ModifyKind::Any when a file was modified
                    if let ModifyKind::Name(rename_mode) = modify_kind {
                        match rename_mode {
                            RenameMode::From => {
                                self.renamed = self.loaded.remove(&path);
                            }
                            RenameMode::To => {
                                log::info!("[Loader] Rename module file: {}", path.display());
                                if let Some(renamed) = self.renamed.take() {
                                    self.loaded.insert(path, renamed);
                                } else {
                                    match Self::create_packed_module(path.clone()) {
                                        Ok(module) => {
                                            self.loaded.insert(path, module);
                                        }
                                        Err(err) => {
                                            log::error!("[Loader] {err}");
                                        }
                                    }
                                }
                                //let renamed = self.renamed.take().expect("Unreachable!");
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn create_packed_module(path: PathBuf) -> Result<PackedModule, Box<dyn std::error::Error>> {
        //TODO check if a module was already loaded
        let bytes = std::fs::read(&path)?;
        PackedModule::create(&bytes, path)
    }

    fn get_packed_plugins(&mut self) -> Vec<PackedModule> {
        let mut vec = vec![];
        self.loaded.drain().for_each(|(_, v)| {
            vec.push(v);
        });
        vec
    }
}

#[derive(Clone)]
pub struct PackedModule {
    pub path: PathBuf,
    pub manifest: Manifest,
    pub config: Config,
    pub module: Vec<u8>,
}

impl PackedModule {
    pub fn create(bytes: &[u8], path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(&mut reader)?;

        let manifest: Manifest = {
            let mut zip_manifest = archive.by_name("manifest.toml")?;
            let mut temp = String::new();
            zip_manifest.read_to_string(&mut temp)?;
            toml::from_str(&temp)
        }?;

        let mut module = Vec::new();
        let mut zip_module = archive.by_name("module.wasm")?;
        zip_module.read_to_end(&mut module)?;

        //archive
        //    .by_name("config/definition.nc")
        //    .map_err(|e| match e {
        //        zip::result::ZipError::FileNotFound => {
        //            ModuleLoaderError::MissingWasmFile(file_name.to_string())
        //        }
        //        other => ModuleLoaderError::Zip(other),
        //    })?;

        Ok(Self {
            path,
            manifest,
            module,
            config: Config::default(),
        })
    }
}
