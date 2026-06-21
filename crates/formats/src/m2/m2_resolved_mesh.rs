use crate::m2::{
    m2_model::M2Model,
    m2_skin::{M2Batch, M2Skin, M2Submesh},
    m2_vertex::M2Vertex,
};

pub struct M2ResolvedMesh {
    pub vertices: Vec<M2Vertex>,
    pub indices: Vec<u32>,

    pub submeshes: Vec<M2Submesh>,
    pub batches: Vec<M2Batch>,

    pub texture_paths: Vec<String>,
    pub texture_lookup: Vec<u16>,
}

impl M2ResolvedMesh {
    pub fn new(model: M2Model, skin: M2Skin) -> Self {
        // Retrieve the vertices referenced in the SKIN file from the M2 model
        let vertices = skin
            .vertices
            .iter()
            .map(|&m2_idx| {
                model
                    .vertices
                    .get(m2_idx as usize)
                    .copied()
                    .expect("SKIN vertex index out of bounds")
            })
            .collect();

        // Just cast all indices to u32 for consistency, they are stored as u16 but we use u32
        let indices = skin.indices.iter().map(|i| *i as u32).collect();

        let submeshes = skin.submeshes;
        let batches = skin.batches;
        let texture_paths = model.texture_paths;
        let texture_lookup = model.texture_lookup;

        Self {
            vertices,
            indices,
            submeshes,
            batches,
            texture_paths,
            texture_lookup,
        }
    }
}
