struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) height: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.height = in.position.y;  // engine Y is up because we're sane
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // change colour with height
    let t = clamp((in.height - 0.0) / 500.0, 0.0, 1.0);
    return mix(
        vec4<f32>(0.1, 0.4, 0.1, 1.0),  // low  - dark green
        vec4<f32>(0.9, 0.9, 0.9, 1.0),  // high - grey/white
        t
    );
}
