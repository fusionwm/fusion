use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use log::info;
use notify::{
    RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
};
use thiserror::Error;
use tokio::sync::mpsc::{Receiver, Sender};
use zip::ZipArchive;

use crate::module::{config::Config, manifest::Manifest};

#[derive(Error, Debug)]
pub enum ModuleLoaderError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("Not a file: {0}")]
    NotAFile(String),

    #[error("Invalid file name: {0}")]
    InvalidFileName(String),

    #[error("Config directory not defined")]
    ConfigDirectoryNotDefined,

    #[error("Missing manifest in module {0}")]
    MissingManifestFile(String),

    #[error("Missing WASM file in module {0}")]
    MissingWasmFile(String),

    #[error("Invalid manifest in module {0}: {1}")]
    InvalidManifest(String, toml::de::Error),
}

pub type ModuleLoaderResult<T> = Result<T, ModuleLoaderError>;

pub struct ModuleLoader {
    error_tx: Sender<ModuleLoaderError>,
    file_rx: Receiver<notify::Result<notify::Event>>,
    loaded: HashMap<PathBuf, PackedModule>,
    renamed: Option<PackedModule>,
}

impl ModuleLoader {
    pub async fn new(error_tx: Sender<ModuleLoaderError>) -> Result<Self, ModuleLoaderError> {
        let config_dir = dirs::config_dir().ok_or(ModuleLoaderError::ConfigDirectoryNotDefined)?;
        let modules_dir = config_dir.join("nethalym/modules");
        if !modules_dir.exists() {
            std::fs::create_dir_all(&modules_dir)?;
        }

        let (file_tx, file_rx) = tokio::sync::mpsc::channel(100);

        let mut instance = Self {
            error_tx,
            file_rx,
            loaded: HashMap::new(),
            renamed: None,
        };

        instance.preload_modules(&modules_dir).await?;

        let thread_dir = modules_dir.clone();
        tokio::task::spawn_blocking(move || {
            let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
            let mut watcher = notify::recommended_watcher(tx).unwrap(); //TODO Error
            watcher
                .watch(thread_dir.as_path(), RecursiveMode::NonRecursive)
                .unwrap(); //TODO Error

            for event in rx {
                let _ = file_tx.blocking_send(event);
            }
        });

        Ok(instance)
    }

    async fn preload_modules(&mut self, modules_dir: &PathBuf) -> ModuleLoaderResult<()> {
        let mut entries = tokio::fs::read_dir(&modules_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "lym") {
                match Self::create_packed_module(&path) {
                    Ok(module) => {
                        //println!("{:#?}", module.manifest);
                        info!("Preload module: {}", module.manifest.name());
                        if let Some(cap) = module.manifest.custom_capabilities() {
                            info!(
                                "[{}] Available capabilities: {:?}",
                                module.manifest.name(),
                                cap
                            );
                        }
                        self.loaded.insert(path, module);
                    }
                    Err(error) => self.error_tx.send(error).await.unwrap(), //TODO Error
                }
            }
        }

        Ok(())
    }

    pub async fn handle_events(&mut self) {
        if let Some(event) = self.file_rx.recv().await {
            let mut event = match event {
                Ok(events) => events,
                Err(error) => {
                    let _ = self.error_tx.send(error.into()).await; //TODO Error
                    return;
                }
            };

            match event.kind {
                notify::EventKind::Create(_) => {
                    for path in event.paths {
                        let metadata = std::fs::metadata(&path)
                            .map_err(async |err| {
                                let _ = self.error_tx.send(err.into()).await;
                            })
                            .ok()
                            .unwrap();

                        if !metadata.is_file() {
                            return;
                        }

                        match Self::create_packed_module(&path) {
                            Ok(module) => {
                                info!("Load module: {}", module.manifest.name());
                                if let Some(cap) = module.manifest.custom_capabilities() {
                                    info!("Available capabilities: {cap:#?}");
                                }
                                self.loaded.insert(path, module);
                            }
                            Err(error) => self.error_tx.send(error).await.unwrap(), //TODO Error
                        }
                    }
                }
                notify::EventKind::Remove(_) => {
                    for path in event.paths {
                        if let Some(module) = self.loaded.remove(&path) {
                            info!("Unload module: {}", module.manifest.name());
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
                                info!("Rename module file: {}", path.display());
                                let renamed = self.renamed.take().expect("Unreachable!");
                                self.loaded.insert(path, renamed);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn create_packed_module(path: impl AsRef<Path>) -> ModuleLoaderResult<PackedModule> {
        let path = path.as_ref();
        //TODO check if a module was already loaded

        let file_name = path
            .file_name()
            .ok_or_else(|| ModuleLoaderError::NotAFile(path.to_string_lossy().to_string()))
            .and_then(|os| {
                os.to_str().ok_or_else(|| {
                    ModuleLoaderError::InvalidFileName(os.to_string_lossy().to_string())
                })
            })?;

        let bytes = std::fs::read(path)?;
        PackedModule::create(&bytes, file_name)
    }

    pub fn get_raw_modules(&mut self) -> Vec<PackedModule> {
        let vec = self.loaded.values().cloned().collect::<Vec<_>>();
        self.loaded.clear();
        vec
    }
}

#[derive(Clone)]
pub struct PackedModule {
    pub path: String,
    pub manifest: Manifest,
    pub config: Config,
    pub module: Vec<u8>,
}

impl PackedModule {
    pub fn create(bytes: &[u8], file_name: &str) -> Result<Self, ModuleLoaderError> {
        let mut reader = Cursor::new(bytes);
        let mut archive = ZipArchive::new(&mut reader)?;

        let manifest: Manifest = {
            let mut zip_manifest = archive.by_name("manifest.toml").map_err(|e| match e {
                zip::result::ZipError::FileNotFound => {
                    ModuleLoaderError::MissingManifestFile(file_name.to_string())
                }
                other => ModuleLoaderError::Zip(other),
            })?;
            let mut temp = String::new();
            zip_manifest.read_to_string(&mut temp)?;
            toml::from_str(&temp)
                .map_err(|err| ModuleLoaderError::InvalidManifest(file_name.to_string(), err))
        }?;

        let mut module = Vec::new();
        {
            let mut zip_module = archive.by_name("module.wasm").map_err(|e| match e {
                zip::result::ZipError::FileNotFound => {
                    ModuleLoaderError::MissingWasmFile(file_name.to_string())
                }
                other => ModuleLoaderError::Zip(other),
            })?;
            zip_module.read_to_end(&mut module)?;
        }

        //archive
        //    .by_name("config/definition.nc")
        //    .map_err(|e| match e {
        //        zip::result::ZipError::FileNotFound => {
        //            ModuleLoaderError::MissingWasmFile(file_name.to_string())
        //        }
        //        other => ModuleLoaderError::Zip(other),
        //    })?;

        Ok(Self {
            path: file_name.to_string(),
            manifest,
            module,
            config: Config::default(),
        })
    }
}
