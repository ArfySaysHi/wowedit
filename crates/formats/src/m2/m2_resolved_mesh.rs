use crate::m2::{m2_model::M2Model, m2_skin::M2Skin, m2_vertex::M2Vertex};

pub struct M2ResolvedMesh {
    pub vertices: Vec<M2Vertex>,
    pub indices: Vec<u32>,
}

impl M2ResolvedMesh {
    pub fn new(model: &M2Model, skin: &M2Skin) -> Self {
        // Retrieve the vertices referenced in the SKIN file from the M2 model
        let vertices = skin
            .vertices
            .iter()
            .map(|m2_idx| model.vertices[*m2_idx as usize])
            .collect();

        // Just cast all indices to u32 for consistency, they are stored as u16 but we use u32
        let indices = skin.indices.iter().map(|i| *i as u32).collect();

        Self { vertices, indices }
    }
}
