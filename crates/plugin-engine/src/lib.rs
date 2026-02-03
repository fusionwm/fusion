#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod config;
pub mod context;
pub mod env;
pub mod general;
pub mod loader;
pub mod manifest;
pub mod table;

mod engine;
pub use engine::*;

pub mod wasm {
    pub use wasmtime::{
        Store,
        component::{Component, Linker, bindgen},
    };
}

pub const FILE_EXTENSION: &str = "fsp";

#[macro_export]
macro_rules! impl_untyped_plugin_binding {
    ($struct:ty) => {
        impl $crate::UntypedPluginBinding for $struct {
            fn type_id(&self) -> core::any::TypeId {
                core::any::TypeId::of::<Self>()
            }

            fn as_any(&self) -> &dyn core::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
                self
            }
        }
    };
}
