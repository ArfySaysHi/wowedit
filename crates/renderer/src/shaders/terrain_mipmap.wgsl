const WORKGROUP_SIZE_X: u32 = 8u;
const WORKGROUP_SIZE_Y: u32 = 8u;

@group(0) @binding(0)
var source_tex: texture_2d<f32>;

@group(0) @binding(1)
var dest_tex: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y)
fn generate_mipmap(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(num_workgroups) num_wgs: vec3<u32>,
) {
    // Get the X and Y coordinates of the output texel we're computing
    let dest_x = global_id.x;
    let dest_y = global_id.y;

    // Calculate the corresponding position in the source texture
    // Since dest is half the size, we multiply by 2 to get source coordinates
    let src_x_base = dest_x * 2u;
    let src_y_base = dest_y * 2u;

    // Sample 4 texels from the source (2x2 box)
    // These are the 4 texels that map to our single output texel
    let sample_00 = textureLoad(source_tex, vec2<u32>(src_x_base, src_y_base), 0);
    let sample_10 = textureLoad(source_tex, vec2<u32>(src_x_base + 1u, src_y_base), 0);
    let sample_01 = textureLoad(source_tex, vec2<u32>(src_x_base, src_y_base + 1u), 0);
    let sample_11 = textureLoad(source_tex, vec2<u32>(src_x_base + 1u, src_y_base + 1u), 0);

    // Average the 4 samples (box filter)
    // This is the simplest mipmap filter - fast but not the highest quality
    let filtered_color = (sample_00 + sample_10 + sample_01 + sample_11) * 0.25f;

    // Write the result to the destination texture
    textureStore(dest_tex, vec2<u32>(dest_x, dest_y), filtered_color);
}
