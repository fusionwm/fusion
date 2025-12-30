use std::any::{Any, TypeId};

use crate::{content::loader::TypedResourceLoader, rendering::material::Material};

pub trait Resource: Any + Send + Sync {
    fn type_id(&self) -> TypeId;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub trait TypedResource<Data>: Resource {
    type ResourceLoader: TypedResourceLoader<Data = Data> + 'static;
}

pub trait GraphicsResource: Resource {
    fn get_material(&self) -> &Material;
}
