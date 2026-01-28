use crate::{ContentManager, Error, TextureHandle};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree},
};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

static HANDLE_ID: AtomicUsize = AtomicUsize::new(0);
fn next_handle_id() -> usize {
    HANDLE_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SvgHandle {
    pub(crate) id: usize,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub struct SvgRequest {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) id: usize,
    pub(crate) pixmap: Pixmap,
    pub(crate) is_static: bool,
}

pub struct SvgData {
    pub tree: Tree,
    pub original_size: (u32, u32),
    pub textures: HashMap<(u32, u32), TextureHandle>,
}

impl ContentManager {
    pub(crate) fn load_svg_from_bytes(&mut self, bytes: &[u8]) -> Result<SvgHandle, Error> {
        let mut options = Options::default();
        options.fontdb_mut().load_system_fonts();
        let tree = Tree::from_data(bytes, &options)?;
        let original_size = (tree.size().width() as u32, tree.size().height() as u32);

        self.svg.push(SvgData {
            tree,
            original_size,
            textures: HashMap::new(),
        });

        Ok(SvgHandle {
            id: next_handle_id(),
            width: original_size.0,
            height: original_size.1,
        })
    }

    pub(crate) fn create_static_texture(
        &self,
        handle: SvgHandle,
        width: u32,
        height: u32,
    ) -> SvgRequest {
        let svg_data = &self.svg[handle.id];
        let mut pixmap = Pixmap::new(width, height)
            .ok_or("Failed to create pixmap")
            .unwrap();

        let scale_x = width as f32 / svg_data.original_size.0 as f32;
        let scale_y = height as f32 / svg_data.original_size.1 as f32;
        resvg::render(
            &svg_data.tree,
            Transform::from_scale(scale_x, scale_y),
            &mut pixmap.as_mut(),
        );

        SvgRequest {
            width,
            height,
            pixmap,
            is_static: true,
            id: handle.id,
        }
    }
}

//pub fn load_svg_from_file(&mut self, path: &str) -> Result<TextureHandle, Error> {
//    let bytes = std::fs::read(path)?;
//    self.load_svg(bytes)
//}
