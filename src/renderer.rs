mod instance;
mod mesh;
mod projection;
mod texture;

use std::path::Path;

use crate::Transform;

use self::projection::*;
pub use self::{instance::*, mesh::*, texture::*};

use pollster::FutureExt as _;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[derive(Debug)]
pub struct DrawCommand<'a> {
    pub texture_attachment: Option<&'a Texture>,
    pub instance_buffer: Option<&'a InstanceBuffer>,
    pub mesh: &'a Mesh,
}

#[derive(Debug)]
pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    projection: Projection,
    default_texture: Texture,
    default_instance_buffer: wgpu::Buffer,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .expect("could not find suitable adapter for rendering");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .expect("could not initialize adapter");

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let (projection, projection_bind_group_layout) =
            Projection::new(&device, size.width, size.height);

        let blank_image = image::DynamicImage::ImageRgba8(image::RgbaImage::new(16, 16));
        let (default_texture, texture_bind_group_layout) =
            Texture::from_image(&device, &queue, &blank_image, 1, 1).unwrap();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[&projection_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let default_instance = Instance::new(Transform::identity(), [0, 0]);
        let default_instance_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance buffer"),
                contents: bytemuck::cast_slice(&[default_instance]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[Vertex::layout(), Instance::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            projection,
            render_pipeline,
            default_texture,
            default_instance_buffer,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        self.projection
            .resize(&self.queue, new_size.width, new_size.height);
    }

    pub fn draw(&mut self, command: &DrawCommand) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command encoder"),
                });

        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, self.projection.bind_group(), &[]);

        if let Some(texture) = command.texture_attachment {
            render_pass.set_bind_group(1, texture.bind_group(), &[]);
        } else {
            render_pass.set_bind_group(1, self.default_texture.bind_group(), &[]);
        }

        let instance_range;
        if let Some(instance_buffer) = command.instance_buffer {
            instance_range = instance_buffer.range();
            render_pass.set_vertex_buffer(1, instance_buffer.buffer().slice(..));
        } else {
            instance_range = 0..1;
            render_pass.set_vertex_buffer(1, self.default_instance_buffer.slice(..));
        }

        render_pass.draw_mesh_instanced(command.mesh, instance_range);

        drop(render_pass);

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn create_mesh(&self, width: f32, height: f32) -> Mesh {
        Mesh::rect(&self.device, width, height)
    }

    pub fn load_texture_atlas<P>(
        &self,
        filepath: P,
        grid_width: u32,
        grid_height: u32,
    ) -> anyhow::Result<Texture>
    where
        P: AsRef<Path>,
    {
        let (texture, _) =
            Texture::from_filepath(&self.device, &self.queue, filepath, grid_width, grid_height)?;

        Ok(texture)
    }

    pub fn create_instance_buffer(&self, instances: &[Instance]) -> InstanceBuffer {
        InstanceBuffer::new(&self.device, instances)
    }

    pub fn update_instance_buffer(&self, instance_buffer: &InstanceBuffer, data: &[Instance]) {
        instance_buffer.update(&self.queue, 0, data)
    }
}
