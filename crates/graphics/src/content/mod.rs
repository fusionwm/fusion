mod font;
mod handle;
mod loader;
mod macros;
mod resource;
mod storage;
mod svg;
mod texture;

pub use handle::Handle;
pub use texture::Texture;

use std::{any::TypeId, collections::HashMap, path::PathBuf, rc::Rc};

use crate::{
    Error,
    content::{
        font::{Font, FontLoader},
        loader::{ResourceLoader, TypedResourceLoader},
        resource::{Resource, TypedResource},
        storage::{ResourceID, ResourceStorage, TypedResourceStorage},
        svg::{Svg, SvgLoader},
        texture::TextureLoader,
    },
    rendering::Gpu,
};

pub struct ContentManager {
    gpu: Rc<Gpu>,
    loaders: HashMap<TypeId, Box<dyn ResourceLoader>>,
    storages: HashMap<TypeId, Box<dyn ResourceStorage>>,
}

impl ContentManager {
    pub fn new(gpu: Rc<Gpu>) -> Self {
        let mut instance = Self {
            loaders: HashMap::default(),
            storages: HashMap::default(),
            gpu: gpu.clone(),
        };

        instance.register_loader::<Texture>(TextureLoader::new(gpu.clone()));
        instance.register_loader::<Font>(FontLoader::new(gpu.clone()));
        instance.register_loader::<Svg>(SvgLoader::new(gpu.clone()));
        instance
    }

    pub fn register_loader<R: Resource>(&mut self, loader: impl ResourceLoader + 'static) {
        let type_id = TypeId::of::<R>();
        assert!(
            !self.loaders.contains_key(&type_id),
            "Loader for resource type {} already registered",
            std::any::type_name::<R>()
        );
        self.loaders.insert(type_id, Box::new(loader));
    }

    pub fn unregister_loader<R: Resource>(&mut self) {
        let type_id = TypeId::of::<R>();
        assert!(
            self.loaders.contains_key(&type_id),
            "Loader for resource type {} not registered",
            std::any::type_name::<R>()
        );
        self.loaders.remove(&type_id);
    }

    //TODO unload resource
    pub fn load_resource<D, R: TypedResource<D>>(
        &mut self,
        path: &str,
        data: D,
    ) -> Result<Handle<R>, Box<dyn std::error::Error>> {
        let loader = {
            let type_id = TypeId::of::<R>();
            self.loaders
                .get(&type_id)
                .and_then(|l| l.as_any().downcast_ref::<R::ResourceLoader>())
        }
        .unwrap();

        if let Some(id) = self.get_resource_id::<R>(path) {
            return Ok(Handle::new(id));
        }

        let bytes = load_asset(path)?;
        let resource = loader.load_resource(&bytes, data)?;

        let storage = self.get_or_add_storage::<R>();
        let id = storage.add(path.to_string(), resource);
        Ok(Handle::new(id))
    }

    fn get_mut_typed_loader<D, R: TypedResource<D>>(&mut self) -> &mut R::ResourceLoader {
        let type_id = TypeId::of::<R>();
        let loader = self.loaders.get_mut(&type_id).unwrap();
        loader
            .as_any_mut()
            .downcast_mut::<R::ResourceLoader>()
            .unwrap()
    }

    fn get_typed_loader<D, R: TypedResource<D>>(&self) -> &R::ResourceLoader {
        let type_id = TypeId::of::<R>();
        let loader = self.loaders.get(&type_id).unwrap();
        loader.as_any().downcast_ref::<R::ResourceLoader>().unwrap()
    }

    fn get_or_add_storage<R: Resource>(&mut self) -> &mut Box<dyn ResourceStorage> {
        self.storages
            .entry(TypeId::of::<R>())
            .or_insert_with(|| Box::new(TypedResourceStorage::<R>::default()))
    }

    fn get_resource_id<R: Resource>(&self, path: &str) -> Option<ResourceID> {
        let type_id = TypeId::of::<R>();
        self.storages
            .get(&type_id)
            .and_then(|s| s.as_any().downcast_ref::<TypedResourceStorage<R>>())
            .and_then(|s| s.get_resource_id_by_path(path))
    }
}

/// # Errors
///
/// This function will return an error if `path` does not already exist.
fn load_asset(path: &str) -> Result<Vec<u8>, Error> {
    let asset_path = get_asset_path().join(path);
    Ok(std::fs::read(asset_path)?)
}

/// # Errors
///
/// This function will return an error if `path` does not already exist.
///
/// If the contents of the file are not valid UTF-8, then an error will also be
/// returned.
fn load_asset_str(path: &str) -> Result<String, Error> {
    let asset_path = get_asset_path().join(path);
    Ok(std::fs::read_to_string(asset_path)?)
}

fn get_asset_path() -> PathBuf {
    // Debug
    #[cfg(debug_assertions)]
    {
        std::env::current_dir().unwrap().join("assets")
        //Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    // Release
    #[cfg(not(debug_assertions))]
    {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("assets")
    }
}

/*
use crate::{
    Error,
    content::svg::{SvgData, SvgRequest},
    rendering::{Gpu, material::Material},
};
use fontdue::{Font, FontSettings};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
pub use svg::SvgHandle;

#[macro_export]
macro_rules! include_asset {
    ($path:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path))
    };
}

#[macro_export]
macro_rules! include_asset_content {
    ($path:expr) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path))
    };
}

static DEFAULT_FONT: std::sync::LazyLock<Arc<Font>> = std::sync::LazyLock::new(|| {
    const BYTES: &[u8; 299_684] = include_asset!("Ubuntu-Regular.ttf");
    let font = Font::from_bytes(BYTES.as_ref(), FontSettings::default()).unwrap();
    Arc::new(font)
});

#[derive(Clone, Debug)]
pub struct FontHandle {
    pub(crate) inner: Arc<Font>,
}

impl PartialEq for FontHandle {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name().eq(&other.inner.name())
    }
}

impl Default for FontHandle {
    fn default() -> Self {
        Self {
            inner: DEFAULT_FONT.clone(),
        }
    }
}

impl AsRef<Font> for FontHandle {
    fn as_ref(&self) -> &Font {
        self.inner.as_ref()
    }
}
*/
