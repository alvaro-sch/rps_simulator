use std::{fmt::Debug, path::Path};

use anyhow::Result;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::renderer::*;

#[derive(Debug, Clone)]
pub struct ContextBuilder {
    pub title: String,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub backends: wgpu::Backends,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = title.into();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = winit::dpi::PhysicalSize::new(width, height);
        self
    }

    pub fn backend(mut self, backends: wgpu::Backends) -> Self {
        self.backends = backends;
        self
    }

    pub fn build(self) -> Result<(Context, EventLoop<()>)> {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title(self.title)
            .with_inner_size(self.size)
            .with_visible(false)
            .build(&event_loop)?;

        let renderer = Renderer::new(&window, self.backends)?;

        Ok((
            Context {
                renderer,
                window,
                close_requested: false,
            },
            event_loop,
        ))
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self {
            title: "no title".into(),
            backends: wgpu::Backends::all(),
            size: winit::dpi::PhysicalSize::new(640, 480),
        }
    }
}

#[derive(Debug)]
pub struct Context {
    renderer: Renderer,
    window: Window,
    close_requested: bool,
}

impl Context {
    pub fn close(&mut self) {
        self.close_requested = true;
    }

    pub fn run<F, A>(mut self, event_loop: EventLoop<()>, app_init: F) -> !
    where
        A: MainLoop + 'static,
        F: FnOnce(&mut Self) -> A,
    {
        let mut app = app_init(&mut self);
        self.window.set_visible(true);

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == self.window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                WindowEvent::Resized(new_size) => self.renderer.resize(*new_size),

                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.renderer.resize(**new_inner_size)
                }

                event => app.event(&mut self, event),
            },

            Event::RedrawRequested(id) if id == self.window.id() => {
                let draw_command = app.draw(&mut self);
                match self.renderer.draw(&draw_command) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(wgpu::SurfaceError::Lost) => self.renderer.resize(self.window.inner_size()),
                    Err(e) => eprintln!("{e:?}"),
                };
            }

            Event::MainEventsCleared => {
                app.update(&mut self);

                if self.close_requested {
                    *control_flow = ControlFlow::Exit;
                }
                self.window.request_redraw();
            }

            _ => {}
        });
    }

    pub fn create_mesh(&self, width: f32, height: f32) -> Mesh {
        Mesh::rect(&self.renderer.device, width, height)
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
        let (texture, _) = Texture::from_filepath(
            &self.renderer.device,
            &self.renderer.queue,
            filepath,
            grid_width,
            grid_height,
        )?;

        Ok(texture)
    }

    pub fn create_instance_buffer(&self, instances: &[Instance]) -> InstanceBuffer {
        InstanceBuffer::new(&self.renderer.device, instances)
    }

    pub fn update_instance_buffer(&self, instance_buffer: &InstanceBuffer, data: &[Instance]) {
        instance_buffer.update(&self.renderer.queue, 0, data)
    }
}

pub trait MainLoop {
    fn update(&mut self, _ctx: &mut Context) {}

    fn event(&mut self, _ctx: &mut Context, _event: &WindowEvent) {}

    fn draw(&mut self, _ctx: &mut Context) -> DrawCommand {
        DrawCommand::Clear(wgpu::Color::BLACK)
    }
}
