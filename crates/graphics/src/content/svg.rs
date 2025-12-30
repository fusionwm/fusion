use std::rc::Rc;

use crate::{
    content::{
        loader::{ResourceLoader, TypedResourceLoader},
        resource::{GraphicsResource, Resource},
    },
    impl_resource,
    rendering::{Gpu, material::Material},
};

use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree},
};

pub struct SvgParams {
    pub width: u32,
    pub height: u32,
}

pub struct SvgLoader {
    gpu: Rc<Gpu>,
}

impl SvgLoader {
    pub const fn new(gpu: Rc<Gpu>) -> Self {
        SvgLoader { gpu }
    }

    fn create_texture(
        &self,
        tree: &Tree,
        original_size: (u32, u32),
        width: u32,
        height: u32,
    ) -> ((u32, u32), Material) {
        let mut pixmap = Pixmap::new(width, height)
            .ok_or("Failed to create pixmap")
            .unwrap();

        let scale_x = width as f32 / original_size.0 as f32;
        let scale_y = height as f32 / original_size.1 as f32;
        resvg::render(
            tree,
            Transform::from_scale(scale_x, scale_y),
            &mut pixmap.as_mut(),
        );

        let material = Material::from_rgba_pixels(
            "svg",
            pixmap.data(),
            (width, height),
            &self.gpu.device,
            &self.gpu.queue,
        );

        ((width, height), material)
    }
}

impl TypedResourceLoader for SvgLoader {
    type Data = SvgParams;

    fn load_resource(
        &self,
        bytes: &[u8],
        params: Self::Data,
    ) -> Result<Box<dyn Resource>, Box<dyn std::error::Error>> {
        let mut options = Options::default();
        options.fontdb_mut().load_system_fonts();
        let tree = Tree::from_data(&bytes, &options)?;
        let original_size = (tree.size().width() as u32, tree.size().height() as u32);
        let (size, material) =
            self.create_texture(&tree, original_size, params.width, params.height);

        Ok(Box::new(Svg { size, material }))
    }
}

impl ResourceLoader for SvgLoader {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct Svg {
    size: (u32, u32),
    material: Material,
}

impl_resource!(Svg, SvgParams, SvgLoader);

impl GraphicsResource for Svg {
    fn get_material(&self) -> &Material {
        &self.material
    }
}
