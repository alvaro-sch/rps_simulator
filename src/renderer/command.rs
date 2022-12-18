use crate::{InstanceBuffer, Mesh, Texture};

#[derive(Debug)]
pub enum DrawCommand<'a> {
    Clear(wgpu::Color),
    DrawMesh(DrawMeshCommand<'a>),
}

#[derive(Debug)]
pub struct DrawMeshCommand<'a> {
    pub texture_attachment: Option<&'a Texture>,
    pub instance_buffer: Option<&'a InstanceBuffer>,
    pub clear_color: Option<wgpu::Color>,
    pub mesh: &'a Mesh,
}
