use rps_simulator::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(&window);

    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

            WindowEvent::Resized(new_size) => renderer.resize(*new_size),

            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(**new_inner_size)
            }

            _ => {}
        },

        Event::RedrawRequested(id) if id == window.id() => match renderer.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            Err(wgpu::SurfaceError::Lost) => renderer.resize(window.inner_size()),
            Err(e) => eprintln!("{e:?}"),
        },

        Event::MainEventsCleared => {
            window.request_redraw();
        }

        _ => {}
    });
}
