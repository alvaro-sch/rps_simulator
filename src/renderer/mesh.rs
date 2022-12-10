use std::mem;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt as _;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

impl Vertex {
    pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub const fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Mesh {
    pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

    pub fn rect(device: &wgpu::Device, width: f32, height: f32) -> Self {
        let buffer = [
            Vertex {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [0.0, height],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [width, height],
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [width, 0.0],
                uv: [1.0, 0.0],
            },
        ];
        let indices = [0u16, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh vertex buffer"),
            contents: bytemuck::cast_slice(&buffer),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn num_indices(&self) -> u32 {
        self.num_indices
    }
}

pub trait DrawMesh<'a> {
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: std::ops::Range<u32>);

    fn draw_mesh(&mut self, mesh: &'a Mesh) {
        <Self as DrawMesh>::draw_mesh_instanced(self, mesh, 0..1);
    }
}

impl<'b, 'a: 'b> DrawMesh<'a> for wgpu::RenderPass<'b> {
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: std::ops::Range<u32>) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), Mesh::INDEX_FORMAT);
        self.draw_indexed(0..mesh.num_indices, 0, instances);
    }
}
