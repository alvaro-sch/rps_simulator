struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct Projection {
    ortho: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = projection.ortho * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}
