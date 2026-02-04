use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, TryRecvError},
    },
    time::Duration,
};

use anyhow::Context;
use notify::{
    RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
};
use zip::ZipArchive;

use crate::{FILE_EXTENSION, config::Config, engine::InnerContext, manifest::Manifest};

#[derive(Copy, Clone)]
pub struct LoaderConfig {
    enable_preload: bool,
    manual_loading: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            enable_preload: true,
            manual_loading: false,
        }
    }
}

impl LoaderConfig {
    #[must_use]
    pub const fn enable_preload(mut self, value: bool) -> Self {
        self.enable_preload = value;
        self
    }

    #[must_use]
    pub const fn manual_loading(mut self, value: bool) -> Self {
        self.manual_loading = value;
        self
    }
}

enum Request {
    GetPlugins,
    LoadPlugin(PathBuf),
}

enum Answer {
    GetPlugins(Vec<FusionPackage>),
}

fn preload_packages<I: InnerContext>(loader: &Arc<Mutex<InnerPluginLoader>>) {
    let mut loader = loader.lock().unwrap();
    log::debug!("[Loader] Preloading packages");
    if let Err(e) = loader.preload_plugins(&I::plugins_path()) {
        log::error!("[Loader] Preload failed: {e:?}");
    }
}

fn run_manual_loader_loop<I: InnerContext>(loader: &Arc<Mutex<InnerPluginLoader>>, preload: bool) {
    log::debug!("[Loader] Starting manual loader loop");

    if preload {
        preload_packages::<I>(loader);
    }

    loop {
        std::thread::sleep(Duration::from_millis(10));
        if let Ok(mut loader) = loader.try_lock() {
            loader.handle_engine_requests();
        }
    }
}

fn run_loader_loop<I: InnerContext>(loader: &Arc<Mutex<InnerPluginLoader>>, preload: bool) {
    log::debug!("[Loader] Starting loader loop");

    if preload {
        preload_packages::<I>(loader);
    }

    loop {
        std::thread::sleep(Duration::from_millis(10));
        if let Ok(mut loader) = loader.try_lock() {
            loader.handle_events();
        }
    }
}

pub(crate) struct PluginLoader {
    _loader: Arc<Mutex<InnerPluginLoader>>,
    request_sender: Sender<Request>,
    answer_receiver: Receiver<Answer>,
}

impl PluginLoader {
    pub fn new<I: InnerContext>(config: LoaderConfig) -> Result<Self, Box<dyn std::error::Error>> {
        log::debug!("[Engine] Initializing loader...");
        let (request_sender, request_receiver) = std::sync::mpsc::channel();
        let (answer_sender, answer_receiver) = std::sync::mpsc::channel();
        let loader = InnerPluginLoader::new(request_receiver, answer_sender, &I::plugins_path())?;
        let loader = Arc::new(Mutex::new(loader));
        let loader_clone = loader.clone();
        std::thread::Builder::new()
            .name("Plugin loader".into())
            .spawn(move || {
                if config.manual_loading {
                    run_manual_loader_loop::<I>(&loader_clone, config.enable_preload);
                } else {
                    run_loader_loop::<I>(&loader_clone, config.enable_preload);
                }
            })?;

        Ok(Self {
            _loader: loader,
            request_sender,
            answer_receiver,
        })
    }

    pub fn get_packages(&mut self) -> Result<Vec<FusionPackage>, Box<dyn std::error::Error>> {
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
    _watcher: notify::RecommendedWatcher,
    watcher_rx: Receiver<Result<notify::Event, notify::Error>>,

    request_receiver: Receiver<Request>,
    answer_sender: Sender<Answer>,

    loaded: HashMap<PathBuf, FusionPackage>,
    renamed: Option<FusionPackage>,
}

impl InnerPluginLoader {
    pub fn new(
        request_receiver: Receiver<Request>,
        answer_sender: Sender<Answer>,
        plugins_path: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(plugins_path, RecursiveMode::NonRecursive)?;

        Ok(Self {
            _watcher: watcher,
            watcher_rx: rx,
            request_receiver,
            answer_sender,
            loaded: HashMap::new(),
            renamed: None,
        })
    }

    fn load_package(&mut self, path: PathBuf) {
        log::debug!("[Loader] Loading package: {}", path.display());
        match Self::create_fusion_package(path.clone()) {
            Ok(module) => {
                log::info!("[Loader] Load plugin: {}", module.manifest.name());
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

    fn preload_plugins(&mut self, plugins_path: &Path) -> anyhow::Result<()> {
        let entries = std::fs::read_dir(plugins_path)?;

        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    if path.is_file() && path.extension().is_some_and(|ext| ext == FILE_EXTENSION) {
                        self.load_package(path);
                    }
                }
                Err(err) => log::error!("[Loader] {err}"),
            }
        }

        Ok(())
    }

    fn handle_engine_requests(&mut self) {
        while let Ok(request) = self.request_receiver.try_recv() {
            match request {
                Request::GetPlugins => {
                    let answer = Answer::GetPlugins(self.get_packed_plugins());
                    if let Err(err) = self.answer_sender.send(answer) {
                        log::error!("[Loader] {err}");
                    }
                }
                Request::LoadPlugin(path) => self.load_package(path),
            }
        }
    }

    fn handle_file_event(&mut self, mut event: notify::Event) {
        match event.kind {
            notify::EventKind::Create(_) => {
                log::debug!("[Watcher] Detected file creation");
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

                    self.load_package(path);
                }
            }
            notify::EventKind::Remove(_) => {
                log::debug!("[Watcher] Detected file removal");
                for path in event.paths {
                    if let Some(module) = self.loaded.remove(&path) {
                        log::info!("[Loader] Unload module: {}", module.manifest.name());
                    }
                }
            }
            notify::EventKind::Modify(modify_kind) => {
                log::debug!("[Watcher] Detected file modification");
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
                                match Self::create_fusion_package(path.clone()) {
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

    fn handle_events(&mut self) {
        self.handle_engine_requests();

        match self.watcher_rx.try_recv() {
            Ok(event) => match event {
                Ok(event) => {
                    self.handle_file_event(event);
                }
                Err(err) => log::error!("[Loader] Error receiving file event: {err}"),
            },
            Err(error) => {
                // Ignore TryRecvError::Empty
                if TryRecvError::Disconnected == error {
                    log::error!("[Loader] Error watching plugins directory: {error}");
                }
            }
        }
    }

    fn create_fusion_package(path: PathBuf) -> anyhow::Result<FusionPackage> {
        //TODO check if a module was already loaded
        let bytes = std::fs::read(&path)?;
        FusionPackage::create(&bytes, path)
    }

    fn get_packed_plugins(&mut self) -> Vec<FusionPackage> {
        let mut vec = vec![];
        self.loaded.drain().for_each(|(_, v)| {
            vec.push(v);
        });
        vec
    }
}

#[derive(Clone)]
pub struct FusionPackage {
    pub path: PathBuf,
    pub manifest: Manifest,
    pub config: Config,
    pub module: Vec<u8>,
}

impl FusionPackage {
    pub fn create(bytes: &[u8], path: PathBuf) -> anyhow::Result<Self> {
        let mut reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(&mut reader)?;

        let manifest: Manifest = {
            let mut zip_manifest = archive
                .by_name("manifest.toml")
                .with_context(|| "Missing 'manifest.toml' file")?;
            let mut temp = String::new();
            zip_manifest.read_to_string(&mut temp)?;
            toml::from_str(&temp)
        }?;

        let mut module = Vec::new();
        let mut zip_module = archive
            .by_name("module.wasm")
            .with_context(|| "Missing 'module.wasm' file")?;
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
