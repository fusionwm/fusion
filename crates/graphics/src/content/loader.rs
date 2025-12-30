use crate::{content::resource::Resource, rendering::Gpu};

pub trait ResourceLoader {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait TypedResourceLoader: ResourceLoader {
    type Data;
    fn load_resource(
        &self,
        bytes: &[u8],
        params: Self::Data,
    ) -> Result<Box<dyn Resource>, Box<dyn std::error::Error>>;
}

pub struct LoadContext<'a> {
    gpu: &'a Gpu,
}
