use std::sync::Weak;

use crate::content::{resource::Resource, storage::ResourceID};

#[derive(Debug)]
pub struct Handle<R: Resource> {
    resource: ResourceID,
    weak_ref: Weak<()>,
    _phantom: std::marker::PhantomData<R>,
}

impl<R: Resource> Clone for Handle<R> {
    fn clone(&self) -> Self {
        Handle {
            resource: self.resource,
            _phantom: std::marker::PhantomData,
            weak_ref: self.weak_ref.clone(),
        }
    }
}

impl<R: Resource> Handle<R> {
    pub const fn new(id: ResourceID) -> Self {
        Handle {
            resource: id,
            _phantom: std::marker::PhantomData,
            weak_ref: Weak::new(),
        }
    }

    pub const fn id(&self) -> ResourceID {
        self.resource
    }
}
