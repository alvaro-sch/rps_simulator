use std::path::Path;

use anyhow::Result;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Texture {
    grid_width: u32,
    grid_height: u32,
    bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn from_filepath<P>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        filepath: P,
        grid_width: u32,
        grid_height: u32,
    ) -> Result<(Self, wgpu::BindGroupLayout)>
    where
        P: AsRef<Path>,
    {
        let bytes = std::fs::read(filepath)?;
        Self::from_bytes(device, queue, &bytes, grid_width, grid_height)
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        grid_width: u32,
        grid_height: u32,
    ) -> Result<(Self, wgpu::BindGroupLayout)> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, grid_width, grid_height)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        grid_width: u32,
        grid_height: u32,
    ) -> Result<(Self, wgpu::BindGroupLayout)> {
        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
            &rgba,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TextureAtlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TextureAtlas grid size"),
            contents: bytemuck::cast_slice(&[grid_width, grid_height]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: grid_buffer.as_entire_binding(),
                },
            ],
        });

        Ok((
            Self {
                grid_width,
                grid_height,
                bind_group,
            },
            bind_group_layout,
        ))
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn grid_size(&self) -> (u32, u32) {
        (self.grid_width, self.grid_height)
    }
}
