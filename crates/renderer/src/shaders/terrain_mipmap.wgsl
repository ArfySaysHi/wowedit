@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0)
    );

    return vec4<f32>(pos[i], 0.0, 1.0);
}

@group(0) @binding(0)
var source_tex: texture_2d<f32>;

@group(0) @binding(1)
var source_sampler: sampler;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(source_tex));
    let uv = pos.xy / size * 2.0; // not actually used directly

    // We sample 4 texels (2x2 box filter)
    let texel = 1.0 / size;

    // Put simply, this will take a sample of 4 texels and average them into a single texel colour
    let color = textureSample(source_tex, source_sampler, vec2<f32>(0.25, 0.25)) +
        textureSample(source_tex, source_sampler, vec2<f32>(0.75, 0.25)) +
        textureSample(source_tex, source_sampler, vec2<f32>(0.25, 0.75)) +
        textureSample(source_tex, source_sampler, vec2<f32>(0.75, 0.75));

    return color * 0.25;
}
