mod command;
mod instance;
mod mesh;
mod projection;
mod texture;

use crate::Transform;

use self::projection::*;
pub use self::{command::*, instance::*, mesh::*, texture::*};

use anyhow::Result;
use pollster::FutureExt as _;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[derive(Debug)]
pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    projection: Projection,
    default_texture: Texture,
    default_instance_buffer: wgpu::Buffer,
}

impl Renderer {
    pub fn new(window: &Window, backends: wgpu::Backends) -> Result<Self> {
        let instance = wgpu::Instance::new(backends);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .ok_or_else(|| anyhow::anyhow!("Could not find suitable adapter"))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()?;

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

        Ok(Self {
            surface,
            device,
            queue,
            config,
            projection,
            render_pipeline,
            default_texture,
            default_instance_buffer,
        })
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

    pub fn clear(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::Color,
    ) {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(*color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
    }

    pub fn draw_mesh(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        command: &DrawMeshCommand,
    ) {
        let load = if let Some(color) = command.clear_color {
            wgpu::LoadOp::Clear(color)
        } else {
            wgpu::LoadOp::Load
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations { load, store: true },
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
    }

    pub fn draw(&mut self, command: &DrawCommand) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command encoder"),
            });

        match command {
            DrawCommand::Clear(color) => self.clear(&view, &mut encoder, color),
            DrawCommand::DrawMesh(command) => self.draw_mesh(&view, &mut encoder, command),
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
