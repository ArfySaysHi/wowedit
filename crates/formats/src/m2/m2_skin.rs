use crate::io::read_u32;
use anyhow::{Result, bail};
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct M2Submesh {
    pub start_index: u32, // offset into the triangle index list
    pub index_count: u32, // how many indices this submesh uses
}

#[derive(Debug)]
pub struct M2Batch {
    pub submesh_index: u16, // indexes into submeshes array
    pub texture_combo_index: u16,
}

#[derive(Debug)]
/// Contains the indices for an m2 model
pub struct M2Skin {
    pub indices: Vec<u32>,
}

/// Skin format is a 4 byte magic b"SKIN"
/// vertices_count<u32> + offset<u32> = the array of u16 vertex
/// remapping indices
/// triangles_count<u32> + offset<u32>
pub fn parse_skin(data: &[u8]) -> Result<M2Skin> {
    let mut r = Cursor::new(data);

    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;

    if &magic != b"SKIN" {
        bail!("Not a skin file");
    }

    let vertex_indices_count = read_u32(&mut r)? as usize;
    let vertex_indices_offset = read_u32(&mut r)? as usize;
    let triangle_count = read_u32(&mut r)? as usize;
    let triangle_offset = read_u32(&mut r)? as usize;
    let _properties_count = read_u32(&mut r)? as usize;
    let _properties_offset = read_u32(&mut r)? as usize;
    // let submesh_count = read_u32(&mut r)? as usize;
    // let submesh_offset = read_u32(&mut r)? as usize;
    // let batch_count = read_u32(&mut r)? as usize;
    // let batch_offset = read_u32(&mut r)? as usize;

    // Read the vertex remapping table
    let mut vertex_indices = Vec::with_capacity(vertex_indices_count);
    r.seek(SeekFrom::Start(vertex_indices_offset as u64))?;
    for _ in 0..vertex_indices_count {
        let mut b = [0u8; 2];
        r.read_exact(&mut b)?;
        vertex_indices.push(u16::from_le_bytes(b));
    }

    // Read the triangle indices and resolve through the remap table
    let mut indices = Vec::with_capacity(triangle_count);
    r.seek(SeekFrom::Start(triangle_offset as u64))?;
    for _ in 0..triangle_count {
        let mut b = [0u8; 2];
        r.read_exact(&mut b)?;
        let skin_idx = u16::from_le_bytes(b) as usize;
        indices.push(vertex_indices[skin_idx] as u32);
    }

    Ok(M2Skin { indices })
}
