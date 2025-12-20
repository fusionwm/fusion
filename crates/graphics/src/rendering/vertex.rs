use glam::Vec3;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexStepMode, vertex_attr_array};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: Vec3,
}

impl Vertex {
    pub fn new(position: Vec3) -> Vertex {
        Self { position }
    }

    pub fn get_layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: [VertexAttribute; 1] = vertex_attr_array![0 => Float32x3];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
