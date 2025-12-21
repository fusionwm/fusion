use wgpu::{
    Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, PresentMode, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages,
};

use crate::Error;
use crate::window::WindowPointer;

pub struct Gpu {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl Gpu {
    pub fn new(dummy: WindowPointer) -> Result<Self, Error> {
        let instance = Instance::new(&InstanceDescriptor::default());
        let surface = instance.create_surface(dummy)?;
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&DeviceDescriptor::default()))?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn create_surface<'window>(
        &self,
        ptr: WindowPointer,
        width: u32,
        height: u32,
    ) -> Result<(Surface<'window>, SurfaceConfiguration), Error> {
        let surface = self.instance.create_surface(ptr)?;

        let caps = surface.get_capabilities(&self.adapter);
        let format = *caps
            .formats
            .iter()
            .find(|&&f| matches!(f, wgpu::TextureFormat::Rgba8Unorm))
            .unwrap_or(&caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };

        self.confugure_surface(&surface, &config);

        Ok((surface, config))
    }

    pub fn confugure_surface(&self, surface: &Surface<'_>, configuration: &SurfaceConfiguration) {
        surface.configure(&self.device, configuration);
    }
}
