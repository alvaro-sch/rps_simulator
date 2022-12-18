use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Instance {
    model: [f32; 9],
    uv_index: [u32; 2],
}

impl Instance {
    pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![2 => Float32x3, 3 => Float32x3, 4 => Float32x3, 5 => Uint32x2];

    pub const fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }

    pub fn new<M>(model: M, uv_index: [u32; 2]) -> Self
    where
        M: Into<[f32; 9]>,
    {
        Self {
            model: model.into(),
            uv_index,
        }
    }
}

#[derive(Debug)]
pub struct InstanceBuffer {
    buffer: wgpu::Buffer,
    range: Range<u32>,
}

impl InstanceBuffer {
    pub fn new(device: &wgpu::Device, instances: &[Instance]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            range: 0..instances.len() as u32,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, offset: wgpu::BufferAddress, data: &[Instance]) {
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data))
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn range(&self) -> Range<u32> {
        self.range.clone()
    }
}
