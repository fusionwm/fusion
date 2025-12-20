use glam::Vec3;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device, BufferUsages};

use crate::rendering::Vertex;

pub struct QuadMesh {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
}

impl QuadMesh {
    pub fn new(device: &Device) -> Self {
        const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let vertices = [
            Vertex::new(Vec3::new(0.0, 0.0, 0.0)), // left top
            Vertex::new(Vec3::new(0.0, 1.0, 0.0)), // left bottom
            Vertex::new(Vec3::new(1.0, 1.0, 0.0)), // right bottom
            Vertex::new(Vec3::new(1.0, 0.0, 0.0)), // right top
        ];
        let vertex_buffer_descriptor = BufferInitDescriptor {
            label: Some("Quad vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        };

        let index_buffer_descriptor = BufferInitDescriptor {
            label: Some("Quad index buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: BufferUsages::INDEX,
        };

        Self {
            vertex_buffer: device.create_buffer_init(&vertex_buffer_descriptor),
            index_buffer: device.create_buffer_init(&index_buffer_descriptor),
        }
    }
}
