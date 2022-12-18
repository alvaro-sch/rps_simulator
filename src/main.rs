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

    let mesh = renderer.create_mesh(60.0, 60.0);
    let texture = renderer
        .load_texture_atlas("assets/rps_atlas.png", 3, 1)
        .unwrap();

    let instances = [(10.0, 0, 0), (80.0, 1, 0), (150.0, 2, 0)]
        .into_iter()
        .map(|(x, uv_x, uv_y)| {
            Instance::new(Transform::identity().translate([x, 10.0]), [uv_x, uv_y])
        })
        .collect::<Vec<_>>();
    let instance_buffer = renderer.create_instance_buffer(&instances);

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

        Event::RedrawRequested(id) if id == window.id() => {
            let draw_command = DrawCommand::DrawMesh(DrawMeshCommand {
                texture_attachment: Some(&texture),
                instance_buffer: Some(&instance_buffer),
                clear_color: None,
                mesh: &mesh,
            });

            match renderer.draw(&draw_command) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(wgpu::SurfaceError::Lost) => renderer.resize(window.inner_size()),
                Err(e) => eprintln!("{e:?}"),
            }
        }

        Event::MainEventsCleared => {
            window.request_redraw();
        }

        _ => {}
    });
}
