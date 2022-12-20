use rps_simulator::*;

struct Simulation {
    mesh: Mesh,
    texture: Texture,
    instance_buffer: InstanceBuffer,
}

impl Simulation {
    pub fn new(ctx: &mut Context) -> Self {
        let mesh = ctx.create_mesh(60.0, 60.0);
        let texture = ctx
            .load_texture_atlas("assets/rps_atlas.png", 3, 1)
            .unwrap();

        let instances = [(10.0, 0, 0), (80.0, 1, 0), (150.0, 2, 0)]
            .into_iter()
            .map(|(x, uv_x, uv_y)| {
                Instance::new(Transform::identity().translate([x, 10.0]), [uv_x, uv_y])
            })
            .collect::<Vec<_>>();
        let instance_buffer = ctx.create_instance_buffer(&instances);

        Self {
            mesh,
            texture,
            instance_buffer,
        }
    }
}

impl MainLoop for Simulation {
    fn draw(&mut self, _ctx: &mut Context) -> DrawCommand {
        DrawCommand::DrawMesh(DrawMeshCommand {
            texture_attachment: Some(&self.texture),
            instance_buffer: Some(&self.instance_buffer),
            clear_color: None,
            mesh: &self.mesh,
        })
    }
}

fn main() {
    let (ctx, event_loop) = ContextBuilder::new()
        .title("rps simulator")
        .build()
        .expect("Failed to create context!");

    ctx.run(event_loop, Simulation::new);
}
