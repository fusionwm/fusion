use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
    Sampler, TextureView,
};

pub struct BindGroupBuilder<'a> {
    entries: Vec<BindGroupEntry<'a>>,
    layout: Option<&'a BindGroupLayout>,
    device: &'a Device,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            entries: vec![],
            layout: None,
            device,
        }
    }

    fn reset(&mut self) {
        self.entries.clear();
    }

    pub fn set_layout(&mut self, layout: &'a BindGroupLayout) {
        self.layout = Some(layout);
    }

    pub fn add_material(&mut self, view: &'a TextureView, sampler: &'a Sampler) {
        self.entries.push(BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: BindingResource::TextureView(view),
        });

        self.entries.push(BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: BindingResource::Sampler(sampler),
        });
    }

    pub fn build(&mut self, label: &'static str) -> BindGroup {
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(label),
            entries: &self.entries,
            layout: self.layout.unwrap(),
        });

        self.reset();

        bind_group
    }
}
