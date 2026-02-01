#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod config;
pub mod context;
pub mod engine;
pub mod env;
pub mod general;
pub mod loader;
pub mod logging;
pub mod manifest;
pub mod table;

pub mod wasm {
    pub use wasmtime::{
        Store,
        component::{Component, Linker, bindgen},
    };
}

pub const FILE_EXTENSION: &str = "fsp";
