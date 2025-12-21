mod svg;

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

enum Request {
    Texture(TextureRequest),
    Svg(SvgRequest),
}

#[derive(Debug, Clone)]
pub enum Handle {
    Texture(TextureHandle),
    Svg(SvgHandle),
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

static HANDLE_ID: AtomicUsize = AtomicUsize::new(0);
fn next_handle_id() -> usize {
    HANDLE_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TextureHandle {
    id: usize,
}

#[derive(Default)]
pub struct ContentManager {
    static_font: HashMap<String, Arc<Font>>,
    static_textures: Vec<Material>,
    svg: Vec<SvgData>,

    queue: Vec<Request>,
}

pub(crate) struct TextureRequest {
    is_static: bool,
    handle_id: usize,
    bytes: &'static [u8],
}

#[allow(dead_code)]
impl ContentManager {
    pub fn include_font(&mut self, bytes: &'static [u8]) -> FontHandle {
        let font = Font::from_bytes(bytes, FontSettings::default()).unwrap();
        let name = font.name().unwrap().to_string();
        let font_handle = Arc::new(font);
        self.static_font.insert(name, font_handle.clone());
        FontHandle { inner: font_handle }
    }

    pub fn static_load_font(&mut self, path: &'static str) -> FontHandle {
        let bytes: &'static [u8] = Box::leak(std::fs::read(path).unwrap().into_boxed_slice());
        let font = Font::from_bytes(bytes, FontSettings::default()).unwrap();
        let name = font.name().unwrap().to_string();
        let font_handle = Arc::new(font);
        self.static_font.insert(name, font_handle.clone());
        FontHandle { inner: font_handle }
    }

    pub(crate) fn get_font(&self, font: &str) -> &Font {
        self.static_font.get(font).unwrap()
    }

    pub fn include_texture(&mut self, bytes: &'static [u8]) -> TextureHandle {
        let handle_id = next_handle_id();
        self.queue.push(Request::Texture(TextureRequest {
            bytes,
            handle_id,
            is_static: true,
        }));

        TextureHandle { id: handle_id }
    }

    pub fn static_load_texture(&mut self, path: &str) -> Result<TextureHandle, Error> {
        let request = TextureRequest {
            handle_id: next_handle_id(),
            bytes: Box::leak(load_asset(path)?.into_boxed_slice()),
            is_static: true,
        };

        let result = Ok(TextureHandle {
            id: request.handle_id,
        });

        self.queue.push(Request::Texture(request));
        result
    }

    pub fn include_svg_as_texture(
        &mut self,
        bytes: &'static [u8],
        width: u32,
        height: u32,
    ) -> SvgHandle {
        let mut svg_handle = self.load_svg_from_bytes(bytes).unwrap();
        svg_handle.width = width;
        svg_handle.height = height;

        let request = self.create_static_texture(svg_handle, width, height);
        self.queue.push(Request::Svg(request));

        svg_handle
    }

    pub(crate) fn dispatch_queue(&mut self, gpu: &Gpu) -> Result<(), Error> {
        self.queue
            .drain(..)
            .try_for_each(|request| -> Result<(), Error> {
                match request {
                    Request::Texture(texture_request) => {
                        let material =
                            Material::from_bytes(texture_request.bytes, &gpu.device, &gpu.queue)?;
                        if texture_request.is_static {
                            self.static_textures.push(material);
                        } else {
                            todo!();
                        }
                    }
                    Request::Svg(svg_request) => {
                        let material = Material::from_rgba_pixels(
                            "svg",
                            svg_request.pixmap.data(),
                            (svg_request.width, svg_request.height),
                            &gpu.device,
                            &gpu.queue,
                        );
                        self.svg.get_mut(svg_request.id).unwrap().textures.insert(
                            (svg_request.width, svg_request.height),
                            TextureHandle {
                                id: next_handle_id(),
                            },
                        );

                        if svg_request.is_static {
                            self.static_textures.push(material);
                        } else {
                            todo!();
                        }
                    }
                }
                Ok(())
            })
    }

    pub(crate) fn get_texture(&self, handle: &Handle) -> &Material {
        match handle {
            Handle::Texture(handle) => self.static_textures.get(handle.id).unwrap(),
            Handle::Svg(handle) => {
                let handle = self
                    .svg
                    .get(handle.id)
                    .unwrap()
                    .textures
                    .get(&(handle.width, handle.height))
                    .unwrap();
                self.static_textures.get(handle.id).unwrap()
            }
        }
    }
}

/// # Errors
///
/// This function will return an error if `path` does not already exist.
pub fn load_asset(path: &str) -> Result<Vec<u8>, Error> {
    let asset_path = get_asset_path().join(path);
    Ok(fs::read(asset_path)?)
}

/// # Errors
///
/// This function will return an error if `path` does not already exist.
///
/// If the contents of the file are not valid UTF-8, then an error will also be
/// returned.
pub fn load_asset_str(path: &str) -> Result<String, Error> {
    let asset_path = get_asset_path().join(path);
    Ok(fs::read_to_string(asset_path)?)
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
