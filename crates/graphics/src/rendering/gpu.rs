use wgpu::{
    Adapter, CompositeAlphaMode, Device, DeviceDescriptor, Instance, InstanceDescriptor,
    PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat,
    TextureUsages,
};

use crate::Error;
use crate::window::WindowPointer;

pub struct Gpu {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,

    //Cache
    pub surface_format: TextureFormat,
    pub alpha_mode: CompositeAlphaMode,
}

impl Gpu {
    pub fn new(dummy: WindowPointer) -> Result<Self, Error> {
        let instance = Instance::new(&InstanceDescriptor::default());
        let surface = instance.create_surface(dummy)?;
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&DeviceDescriptor::default()))?;

        let caps = surface.get_capabilities(&adapter);

        // Cache values
        let surface_format = *caps
            .formats
            .iter()
            .find(|&&f| matches!(f, wgpu::TextureFormat::Rgba8Unorm))
            .unwrap_or(&caps.formats[0]);

        let alpha_mode = caps.alpha_modes[0];

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface_format,
            alpha_mode,
        })
    }

    pub fn create_surface<'window>(
        &self,
        ptr: WindowPointer,
        width: u32,
        height: u32,
    ) -> Result<(Surface<'window>, SurfaceConfiguration), Error> {
        let surface = self.instance.create_surface(ptr)?;

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 1,
            alpha_mode: self.alpha_mode,
            view_formats: vec![],
        };

        self.confugure_surface(&surface, &config);

        Ok((surface, config))
    }

    pub fn confugure_surface(&self, surface: &Surface<'_>, configuration: &SurfaceConfiguration) {
        surface.configure(&self.device, configuration);
    }
}
