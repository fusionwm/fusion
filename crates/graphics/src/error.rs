use image::ImageError;
use thiserror::Error;
use wayland_client::{ConnectError, DispatchError};
use wgpu::{CreateSurfaceError, RequestAdapterError, RequestDeviceError, SurfaceError};

#[derive(Error, Debug)]
pub enum Error {
    //Wayland
    #[error("{0}")]
    Connect(#[from] ConnectError),
    #[error("{0}")]
    Dispatch(#[from] DispatchError),

    //Wgpu
    #[error("{0}")]
    CreateSurface(#[from] CreateSurfaceError),
    #[error("{0}")]
    RequestAdapter(#[from] RequestAdapterError),
    #[error("{0}")]
    RequestDevice(#[from] RequestDeviceError),
    #[error("{0}")]
    Surface(#[from] SurfaceError),

    // Images
    #[error("{0}")]
    Image(#[from] ImageError),

    #[error("{0}")]
    Svg(#[from] resvg::usvg::Error),

    //Std
    #[error("{0}")]
    IO(#[from] std::io::Error),

    #[error("Display pointer is null")]
    DisplayNullPointer,

    #[error("{0}")]
    LockFailed(String),

    #[error("Width must be >= 0. Actual value {0}")]
    NegativeWidth(i32),

    #[error("Height must be >= 0. Actual value {0}")]
    NegativeHeight(i32)
}
