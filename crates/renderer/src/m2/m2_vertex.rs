use formats::m2::m2_vertex::M2Vertex;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct M2GpuVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl M2GpuVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<M2GpuVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl From<M2Vertex> for M2GpuVertex {
    fn from(v: M2Vertex) -> Self {
        Self {
            // WoW model space: X=south, Z=up → engine space: X=east, Y=up
            position: [-v.position[1], v.position[2], -v.position[0]],
            normal: [-v.normal[1], v.normal[2], -v.normal[0]],
            uv: v.tex_coords[0],
        }
    }
}
