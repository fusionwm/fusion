use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, Device, SamplerBindingType, ShaderStages, TextureSampleType,
    TextureViewDimension,
};

pub struct BindGroupLayoutBuilder<'a> {
    entries: Vec<BindGroupLayoutEntry>,
    device: &'a Device,
}

impl<'a> BindGroupLayoutBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            entries: vec![],
            device,
        }
    }

    fn reset(&mut self) {
        self.entries.clear();
    }

    pub fn add_material(&mut self) {
        self.entries.push(BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });

        self.entries.push(BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        });

        self.entries.push(BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }

    pub fn build(&mut self, label: &'static str) -> BindGroupLayout {
        let layout = self
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &self.entries,
            });

        self.reset();
        layout
    }
}
