pub mod bind_group;
pub mod bind_group_layout;
pub mod commands;
pub mod material;
pub mod mesh;

mod gpu;
mod instance;
mod text;
mod vertex;

pub use gpu::Gpu;

use crate::error::Error;
use crate::rendering::bind_group_layout::BindGroupLayoutBuilder;
use crate::rendering::material::Material;
use crate::rendering::mesh::QuadMesh;
use crate::rendering::text::FontAtlasSet;
use crate::rendering::vertex::Vertex;
use crate::{include_asset_content, load_asset_str};
use glam::Mat4;
use std::collections::HashMap;
use wgpu::{
    BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Face,
    FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, TextureViewDescriptor, VertexState,
};

pub struct Renderer {
    render_pipeline: RenderPipeline,
    mesh: QuadMesh,
    material: Material,
    fonts: HashMap<String, FontAtlasSet>,
    projection: Mat4,
}

impl Renderer {
    pub fn new(gpu: &Gpu, shader: Option<&str>, surface: &Surface) -> Result<Self, Error> {
        let mut builder = BindGroupLayoutBuilder::new(&gpu.device);
        builder.add_material();
        let layout = builder.build("Default");

        let (shader, shader_label) = if let Some(shader) = shader {
            (load_asset_str(shader)?, shader)
        } else {
            (include_asset_content!("shader.wgsl").to_string(), "Default")
        };

        let shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(shader_label),
            source: ShaderSource::Wgsl(shader.into()),
        });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&layout],
                //push_constant_ranges: &[],
                immediate_size: 0, //TODO
            });

        let caps = surface.get_capabilities(&gpu.adapter);
        let format = caps
            .formats
            .iter()
            .find(|&&f| matches!(f, wgpu::TextureFormat::Rgba8Unorm))
            .unwrap_or(&caps.formats[0]);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[Vertex::get_layout()],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        //format: surface.get_capabilities(&gpu.adapter).formats[0],
                        format: *format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                //multiview: None,
                cache: None,
                multiview_mask: None, //TODO
            });

        Ok(Self {
            render_pipeline,
            mesh: QuadMesh::new(&gpu.device),
            material: Material::default(&gpu.device, &gpu.queue),
            fonts: HashMap::default(),
            projection: Mat4::IDENTITY,
        })
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        surface: &Surface,
        commands: &mut commands::CommandBuffer,
        window_width: f32,
        window_height: f32,
    ) -> Result<(), Error> {
        let texture = surface.get_current_texture()?;
        let image_view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let color_attachment = RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
            depth_slice: None,
        };

        let render_pass_descriptor = RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None, //TODO
        };

        let mut command_encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.projection =
            Mat4::orthographic_rh_gl(0.0, window_width, window_height, 0.0, -1.0, 1.0);

        {
            for set in self.fonts.values_mut() {
                set.clear_storage_buffers();
            }
            self.material.clear_storage_buffer();

            let mut renderpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            renderpass.set_pipeline(&self.render_pipeline);
            renderpass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            renderpass.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint16);

            commands.iter_mut().for_each(|(content, group)| {
                group.prepare_frame(self, content, gpu, &mut renderpass);
            });
        }

        gpu.queue.submit(std::iter::once(command_encoder.finish()));
        texture.present();

        Ok(())
    }
}

/*
pub mod bind_group;
pub mod bind_group_layout;
pub mod commands;
pub mod material;
pub mod mesh;

mod gpu;
mod instance;
pub mod text;
mod vertex;

pub use gpu::Gpu;

use crate::error::Error;
use crate::rendering::bind_group_layout::BindGroupLayoutBuilder;
use crate::rendering::material::Material;
use crate::rendering::mesh::QuadMesh;
use crate::rendering::text::FontAtlasSet;
use crate::rendering::vertex::Vertex;
use glam::Mat4;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use wgpu::{
    BlendState, Color, ColorTargetState, ColorWrites, CommandEncoder, CommandEncoderDescriptor,
    Face, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, StoreOp, Surface,
    TextureViewDescriptor, VertexState,
};

static RENDER_PIPELINE_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderPipelineHandle(u32);

pub struct Renderer_ {
    projection: Mat4,
    pipelines: HashMap<RenderPipelineHandle, RenderPipeline>,
    mesh: QuadMesh, //Why??
}

impl Renderer_ {
    pub fn new(gpu: &Gpu) -> Self {
        let mesh = QuadMesh::new(&gpu.device);
        Self {
            mesh,
            projection: Mat4::IDENTITY,
            pipelines: HashMap::new(),
        }
    }

    pub fn create_render_pipeline<'a>(
        &mut self,
        gpu: &Gpu,
        shader: impl Into<Cow<'a, str>>,
        shader_label: &str,
        surface: &Surface,
    ) -> RenderPipelineHandle {
        let mut builder = BindGroupLayoutBuilder::new(&gpu.device);
        builder.add_material();
        let layout = builder.build("Default");

        let shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(shader_label),
            source: ShaderSource::Wgsl(shader.into()),
        });

        //TODO
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&layout],
                //push_constant_ranges: &[],
                immediate_size: 0, //TODO
            });

        let caps = surface.get_capabilities(&gpu.adapter);
        let format = caps
            .formats
            .iter()
            .find(|&&f| matches!(f, wgpu::TextureFormat::Rgba8Unorm))
            .unwrap_or(&caps.formats[0]);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[Vertex::get_layout()],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        //format: surface.get_capabilities(&gpu.adapter).formats[0],
                        format: *format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                //multiview: None,
                cache: None,
                multiview_mask: None, //TODO
            });

        let id = RenderPipelineHandle(RENDER_PIPELINE_ID.fetch_add(1, Ordering::Relaxed));
        self.pipelines.insert(id, render_pipeline);
        id
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        surface: &Surface,
        commands: &mut commands::CommandBuffer,
        window_width: f32,
        window_height: f32,
    ) -> Result<(), Error> {
        let texture = surface.get_current_texture()?;
        let image_view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let color_attachment = RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
            depth_slice: None,
        };

        let render_pass_descriptor = RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None, //TODO
        };

        let mut command_encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let render_phase = Render {
            command_encoder,
            projection: Mat4::orthographic_rh_gl(0.0, window_width, window_height, 0.0, -1.0, 1.0),
            pipelines: &self.pipelines,
            descriptor: render_pass_descriptor,
            mesh: &self.mesh,
        };

        {
            let mut renderpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            renderpass.set_pipeline(&self.render_pipeline);
            renderpass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            renderpass.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint16);

            commands.iter_mut().for_each(|(content, group)| {
                group.prepare_frame(self, content, gpu, &mut renderpass);
            });
        }

        gpu.queue.submit(std::iter::once(command_encoder.finish()));
        texture.present();

        Ok(())
    }
}

pub struct Render<'a> {
    command_encoder: CommandEncoder,
    projection: Mat4,
    pipelines: &'a HashMap<RenderPipelineHandle, RenderPipeline>,
    descriptor: RenderPassDescriptor<'a>,
    mesh: &'a QuadMesh,

    current_pipeline: RenderPipelineHandle,
}

impl Render<'_> {
    pub fn set_pipeline(&mut self, handle: RenderPipelineHandle) -> RenderPass<'_> {
        self.current_pipeline = handle;
        let mut renderpass = self.command_encoder.begin_render_pass(&self.descriptor);
        renderpass.set_pipeline(&self.pipelines[&handle]);
        renderpass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        renderpass.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint16);
        //renderpass.set_bind_group(0, &material.bind_group, &[]);
        renderpass
    }

    fn draw_instances(&self, material: &mut Material) {
        material.resize_buffer_if_needed(gpu, self.re);
        self.write_instance_buffer(gpu);
        renderpass.draw_indexed(0..6, 0, 0..self.instances.len() as u32);
    }
}

pub struct Renderer {
    render_pipeline: RenderPipeline,
    mesh: QuadMesh,
    material: Material,
    fonts: HashMap<String, FontAtlasSet>,
    projection: Mat4,
}

impl Renderer {
    pub fn new(gpu: &Gpu, shader: Option<&str>, surface: &Surface) -> Result<Self, Error> {
        let mut builder = BindGroupLayoutBuilder::new(&gpu.device);
        builder.add_material();
        let layout = builder.build("Default");

        let (shader, shader_label) = if let Some(shader) = shader {
            (load_asset_str(shader)?, shader)
        } else {
            (include_asset_content!("shader.wgsl").to_string(), "Default")
        };

        let shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(shader_label),
            source: ShaderSource::Wgsl(shader.into()),
        });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&layout],
                //push_constant_ranges: &[],
                immediate_size: 0, //TODO
            });

        let caps = surface.get_capabilities(&gpu.adapter);
        let format = caps
            .formats
            .iter()
            .find(|&&f| matches!(f, wgpu::TextureFormat::Rgba8Unorm))
            .unwrap_or(&caps.formats[0]);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[Vertex::get_layout()],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        //format: surface.get_capabilities(&gpu.adapter).formats[0],
                        format: *format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                //multiview: None,
                cache: None,
                multiview_mask: None, //TODO
            });

        Ok(Self {
            render_pipeline,
            mesh: QuadMesh::new(&gpu.device),
            material: Material::default(&gpu.device, &gpu.queue),
            fonts: HashMap::default(),
            projection: Mat4::IDENTITY,
        })
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        surface: &Surface,
        commands: &mut commands::CommandBuffer,
        window_width: f32,
        window_height: f32,
    ) -> Result<(), Error> {
        let texture = surface.get_current_texture()?;
        let image_view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let color_attachment = RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
            depth_slice: None,
        };

        let render_pass_descriptor = RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None, //TODO
        };

        let mut command_encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.projection =
            Mat4::orthographic_rh_gl(0.0, window_width, window_height, 0.0, -1.0, 1.0);

        {
            self.material.clear_storage_buffer();

            let mut renderpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            renderpass.set_pipeline(&self.render_pipeline);
            renderpass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            renderpass.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint16);

            commands.iter_mut().for_each(|(content, group)| {
                group.prepare_frame(self, content, gpu, &mut renderpass);
            });
        }

        gpu.queue.submit(std::iter::once(command_encoder.finish()));
        texture.present();

        Ok(())
    }
}
*/
