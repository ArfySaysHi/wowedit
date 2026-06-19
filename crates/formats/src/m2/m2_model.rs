use crate::m2::m2_vertex::M2Vertex;

pub struct M2Model {
    pub vertices: Vec<M2Vertex>,
    pub texture_lookup: Vec<u16>,
    pub texture_paths: Vec<String>,
}
