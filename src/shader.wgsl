struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct InstanceInput {
    @location(2) model_0: vec3<f32>,
    @location(3) model_1: vec3<f32>,
    @location(4) model_3: vec3<f32>,
    @location(5) uv_index: vec2<u32>,
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
fn vertex(v_in: VertexInput, i_in: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        vec4<f32>(i_in.model_0,  0.0),
        vec4<f32>(i_in.model_1,  0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(i_in.model_3,  1.0),
    );

    var out: VertexOutput;

    out.clip_position = projection.ortho * model * vec4<f32>(v_in.position, 0.0, 1.0);
    out.uv = v_in.uv + vec2<f32>(i_in.uv_index);

    return out;
}

struct TextureData {
    grid_size: vec2<u32>,
}

@group(1) @binding(0)
var t_view: texture_2d<f32>;
@group(1) @binding(1)
var t_sampler: sampler;
@group(1) @binding(2)
var<uniform> t_data: TextureData;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let grid_size = vec2<f32>(t_data.grid_size);

    let uv = in.uv / grid_size;

    return textureSample(t_view, t_sampler, uv);
}
