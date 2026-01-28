use crate::{
    error::Error,
    rendering::{
        Gpu, bind_group::BindGroupBuilder, bind_group_layout::BindGroupLayoutBuilder,
        instance::InstanceData,
    },
};
use image::GenericImageView;
use wgpu::{
    AddressMode, BindGroup, Buffer, BufferDescriptor, Device, Extent3d, FilterMode, Origin3d,
    Queue, RenderPass, SamplerDescriptor, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor,
};

pub struct MaterialDescriptor<'a> {
    pub label: &'static str,
    pub pixels: &'a [u8],
    pub size: (u32, u32),
    pub format: TextureFormat,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
}

pub struct Material {
    pub bind_group: BindGroup,
    pub instances: Vec<InstanceData>,
    wgpu_buffer: Buffer,
    wgpu_buffer_len: usize,

    drawn: u32,
}

const MAX_INSTANCES: usize = 2048;

impl Material {
    pub(crate) fn from_pixels(desc: &MaterialDescriptor, device: &Device, queue: &Queue) -> Self {
        let texture_size = Extent3d {
            width: desc.size.0,
            height: desc.size.1,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = TextureDescriptor {
            label: Some(desc.label),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: desc.format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[desc.format],
        };

        let texture = device.create_texture(&texture_descriptor);
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            desc.pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(desc.size.0 * 4), //TODO: fix unknown format
                rows_per_image: Some(desc.size.1),
            },
            texture_size,
        );
        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler_descriptor = SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: desc.mag_filter,
            min_filter: desc.min_filter,
            ..Default::default()
        };
        let sampler = device.create_sampler(&sampler_descriptor);

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Storage Buffer"),
            size: (MAX_INSTANCES * size_of::<InstanceData>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let mut layout_builder = BindGroupLayoutBuilder::new(device);
        layout_builder.add_material();
        let layout = layout_builder.build("Material");

        let mut builder = BindGroupBuilder::new(device);
        builder.set_layout(&layout);
        builder.add_material(&view, &sampler, buffer.as_entire_buffer_binding());
        let bind_group = builder.build(desc.label);

        Material {
            bind_group,
            instances: vec![],
            wgpu_buffer: buffer,
            wgpu_buffer_len: MAX_INSTANCES,
            drawn: 0,
        }
    }

    pub(crate) fn from_rgba_pixels(
        label: &'static str,
        pixels: &[u8],
        size: (u32, u32),
        device: &Device,
        queue: &Queue,
    ) -> Self {
        Self::from_pixels(
            &MaterialDescriptor {
                label,
                pixels,
                size,
                format: TextureFormat::Rgba8Unorm,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Nearest,
            },
            device,
            queue,
        )
    }

    pub fn default(device: &Device, queue: &Queue) -> Self {
        Self::from_rgba_pixels("Default", &[255, 255, 255, 255], (1, 1), device, queue)
    }

    pub fn from_bytes(bytes: &[u8], device: &Device, queue: &Queue) -> Result<Self, Error> {
        let image = image::load_from_memory(bytes)?;
        let converted = image.to_rgba8();
        let size = image.dimensions();
        Ok(Self::from_rgba_pixels(
            "texture", &converted, size, device, queue,
        ))
    }

    pub fn push(&mut self, instance: InstanceData) {
        self.instances.push(instance);
    }

    fn create_storage_buffer(&mut self, gpu: &Gpu, size: usize) {
        let buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Storage Buffer"),
            size: (size * size_of::<InstanceData>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        self.wgpu_buffer = buffer;
        self.wgpu_buffer_len = size;
    }

    pub(crate) fn resize_buffer_if_needed(&mut self, gpu: &Gpu, renderpass: &mut RenderPass) {
        if self.instances.capacity() > self.wgpu_buffer_len {
            self.create_storage_buffer(gpu, self.instances.capacity());
            //renderpass.set_vertex_buffer(1, self.wgpu_buffer.slice(..));
        }
    }

    pub(crate) fn write_instance_buffer(&self, gpu: &Gpu) {
        gpu.queue
            .write_buffer(&self.wgpu_buffer, 0, bytemuck::cast_slice(&self.instances));
    }

    pub fn draw_instances(&mut self, gpu: &Gpu, renderpass: &mut RenderPass, count: u32) {
        self.resize_buffer_if_needed(gpu, renderpass);
        self.write_instance_buffer(gpu);
        renderpass.set_bind_group(0, &self.bind_group, &[]);
        renderpass.set_vertex_buffer(1, self.wgpu_buffer.slice(..));
        renderpass.draw_indexed(0..6, 0, self.drawn..self.drawn + count);
        self.drawn += count;
    }

    pub fn clear_storage_buffer(&mut self) {
        self.drawn = 0;
        self.instances.clear();
    }
}
