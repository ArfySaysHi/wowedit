use crate::m2::{M2Material, m2_vertex::M2Vertex};

pub struct M2Model {
    pub vertices: Vec<M2Vertex>,
    pub texture_paths: Vec<String>,
    pub texture_lookup: Vec<u16>,
    pub materials: Vec<M2Material>,
}
