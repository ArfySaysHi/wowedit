#[derive(Debug, Copy, Clone)]
pub struct M2Vertex {
    pub position: [f32; 3],
    pub bone_weights: [u8; 4],
    pub bone_indices: [u8; 4],
    pub normal: [f32; 3],
    pub tex_coords: [[f32; 2]; 2],
}
