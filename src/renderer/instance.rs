use bytemuck::{Pod, Zeroable};

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
