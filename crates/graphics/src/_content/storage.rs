use std::collections::HashMap;

use slotmap::{SlotMap, new_key_type};

use crate::content::resource::Resource;

new_key_type! { pub struct ResourceID; }

pub trait ResourceStorage {
    fn remove(&mut self, id: ResourceID);
    fn add(&mut self, path: String, resource: Box<dyn Resource>) -> ResourceID;
    fn get_loaded(&self, id: ResourceID) -> &dyn Resource;
    fn get_resource_id_by_path(&self, path: &str) -> Option<ResourceID>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct TypedResourceStorage<R: Resource> {
    loaded: HashMap<String, ResourceID>,
    resources: SlotMap<ResourceID, R>,
}

impl<R: Resource> Default for TypedResourceStorage<R> {
    fn default() -> Self {
        Self {
            loaded: HashMap::new(),
            resources: SlotMap::with_key(),
        }
    }
}

impl<R: Resource> ResourceStorage for TypedResourceStorage<R> {
    fn remove(&mut self, id: ResourceID) {
        self.resources.remove(id);
        self.loaded.retain(|_, &mut rid| rid != id);
    }

    fn add(&mut self, path: String, resource: Box<dyn Resource>) -> ResourceID {
        let any = resource.into_any();
        let resource = *any.downcast::<R>().unwrap();
        let id = self.resources.insert(resource);
        self.loaded.insert(path, id);
        id
    }

    fn get_loaded(&self, id: ResourceID) -> &dyn Resource {
        self.resources.get(id).unwrap()
    }

    fn get_resource_id_by_path(&self, path: &str) -> Option<ResourceID> {
        self.loaded.get(path).copied()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<R: Resource> TypedResourceStorage<R> {
    pub fn get_resource(&self, id: ResourceID) -> Option<&R> {
        self.resources.get(id)
    }

    pub fn get_mut_resource(&mut self, id: ResourceID) -> Option<&mut R> {
        self.resources.get_mut(id)
    }
}
