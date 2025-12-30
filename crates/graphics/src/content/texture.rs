use std::rc::Rc;

use crate::{
    content::{
        loader::{ResourceLoader, TypedResourceLoader},
        resource::{GraphicsResource, Resource},
    },
    impl_resource,
    rendering::{Gpu, material::Material},
};

pub struct TextureLoader {
    gpu: Rc<Gpu>,
}

impl TextureLoader {
    pub const fn new(gpu: Rc<Gpu>) -> Self {
        Self { gpu }
    }
}

impl TypedResourceLoader for TextureLoader {
    type Data = ();

    fn load_resource(
        &self,
        bytes: &[u8],
        _params: Self::Data,
    ) -> Result<Box<dyn Resource>, Box<dyn std::error::Error>> {
        let material = Material::from_bytes(bytes, &self.gpu.device, &self.gpu.queue)?;
        let texture = Texture { material };
        Ok(Box::new(texture))
    }
}

impl ResourceLoader for TextureLoader {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct Texture {
    material: Material,
}

impl_resource!(Texture, (), TextureLoader);

impl GraphicsResource for Texture {
    fn get_material(&self) -> &Material {
        &self.material
    }
}
