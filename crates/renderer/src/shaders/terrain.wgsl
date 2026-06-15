struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var terrain_textures: texture_2d_array<f32>;
@group(1) @binding(1) var terrain_sampler: sampler;

@group(2) @binding(0) var alpha_maps: texture_2d_array<f32>;
@group(2) @binding(1) var alpha_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) chunk_uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) chunk_uv: vec2<f32>,
}

struct ChunkMaterial {
    layer_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    texture_ids: vec4<u32>,
}
@group(2) @binding(2) var<uniform> material: ChunkMaterial;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.chunk_uv = in.chunk_uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tiled_uv = in.uv;
    var col = textureSample(terrain_textures, terrain_sampler, tiled_uv, material.texture_ids.x);

    if material.layer_count > 1u {
        let a1 = textureSample(alpha_maps, alpha_sampler, in.chunk_uv, 0).r;
        let t1 = textureSample(terrain_textures, terrain_sampler, tiled_uv, material.texture_ids.y);
        col = mix(col, t1, a1);
    }
    if material.layer_count > 2u {
        let a2 = textureSample(alpha_maps, alpha_sampler, in.chunk_uv, 1).r;
        let t2 = textureSample(terrain_textures, terrain_sampler, tiled_uv, material.texture_ids.z);
        col = mix(col, t2, a2);
    }
    if material.layer_count > 3u {
        let a3 = textureSample(alpha_maps, alpha_sampler, in.chunk_uv, 2).r;
        let t3 = textureSample(terrain_textures, terrain_sampler, tiled_uv, material.texture_ids.w);
        col = mix(col, t3, a3);
    }

    return col;
}

