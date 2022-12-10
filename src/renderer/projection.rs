use wgpu::util::DeviceExt as _;

#[derive(Debug)]
pub struct Projection {
    data: [f32; 16],
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Projection {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> (Self, wgpu::BindGroupLayout) {
        let data = Self::create_projection_matrix(width, height);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Projection buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Projection bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Projection Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        (
            Self {
                data,
                buffer,
                bind_group,
            },
            bind_group_layout,
        )
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.data = Self::create_projection_matrix(width, height);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn create_projection_matrix(width: u32, height: u32) -> [f32; 16] {
        let (l, r) = (0.0, width as f32);
        let (t, b) = (0.0, height as f32);

        #[rustfmt::skip]
        let mut matrix = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        matrix[0] = 2.0 / (r - l);
        matrix[5] = 2.0 / (t - b);

        matrix[12] = -(r + l) / (r - l);
        matrix[13] = -(t + b) / (t - b);

        matrix
    }
}
