use crate::io::{read_f32, read_u8, read_u16, read_u32};
use anyhow::{Result, bail};
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct M2Lookup {
    pub size: u32,
    pub offset: u32,
}

#[derive(Debug)]
pub struct M2SkinHeader {
    pub magic: [u8; 4],
    pub vertices: M2Lookup,
    pub indices: M2Lookup,
    pub bones: M2Lookup,
    pub submeshes: M2Lookup,
    pub batches: M2Lookup,
    pub bone_count: u32,
}

#[derive(Debug)]
pub struct M2Skin {
    /// These are not actual vertices, they are references to the index of vertices in the M2 file,
    /// instead of rendering all vertices from the m2, we render a smaller number from here
    pub vertices: Vec<u16>,
    /// There are indices that connect the vertices in the skin file together into triangles
    pub indices: Vec<u16>,
    /// TODO: Clarify
    pub submeshes: Vec<M2Submesh>,
    /// TODO: Clarify
    pub batches: Vec<M2Batch>,
}

#[derive(Debug)]
pub struct M2Submesh {
    pub id: u16,
    pub level: u16,

    pub vertex_start: u16,
    pub vertex_count: u16,

    pub index_start: u16,
    pub index_count: u16,

    pub bone_count: u16,
    pub bone_combo_index: u16,
    pub bone_influences: u16,
    pub center_bone_index: u16,

    pub center: [f32; 3],
    pub sort_center: [f32; 3],
    pub sort_radius: f32,
}

#[derive(Debug)]
pub struct M2Batch {
    pub flags: u8,
    pub priority: u8,
    pub shader_id: u16,

    pub submesh_index: u16,
    pub material_index: u16,

    pub texture_index: u16,
    pub tex_unit_index: u16,

    pub transparency_index: u16,
    pub texture_anim_index: u16,
}

fn read_lookup(r: &mut Cursor<&[u8]>) -> Result<M2Lookup> {
    Ok(M2Lookup {
        size: read_u32(r)?,
        offset: read_u32(r)?,
    })
}

fn parse_header(r: &mut Cursor<&[u8]>) -> Result<M2SkinHeader> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;

    if &magic != b"SKIN" {
        bail!("Not a SKIN file");
    }

    Ok(M2SkinHeader {
        magic,
        vertices: read_lookup(r)?,
        indices: read_lookup(r)?,
        bones: read_lookup(r)?,
        submeshes: read_lookup(r)?,
        batches: read_lookup(r)?,
        bone_count: read_u32(r)?,
    })
}

fn read_submeshes(header: &M2SkinHeader, r: &mut Cursor<&[u8]>) -> Result<Vec<M2Submesh>> {
    r.seek(SeekFrom::Start(header.submeshes.offset as u64))?;

    let mut submeshes = Vec::with_capacity(header.submeshes.size as usize);

    for _ in 0..header.submeshes.size {
        submeshes.push(M2Submesh {
            id: read_u16(r)?,
            level: read_u16(r)?,
            vertex_start: read_u16(r)?,
            vertex_count: read_u16(r)?,
            index_start: read_u16(r)?,
            index_count: read_u16(r)?,
            bone_count: read_u16(r)?,
            bone_combo_index: read_u16(r)?,
            bone_influences: read_u16(r)?,
            center_bone_index: read_u16(r)?,

            center: [read_f32(r)?, read_f32(r)?, read_f32(r)?],
            sort_center: [read_f32(r)?, read_f32(r)?, read_f32(r)?],
            sort_radius: read_f32(r)?,
        });
    }

    Ok(submeshes)
}

fn read_batches(header: &M2SkinHeader, r: &mut Cursor<&[u8]>) -> Result<Vec<M2Batch>> {
    r.seek(SeekFrom::Start(header.batches.offset as u64))?;

    let mut batches = Vec::with_capacity(header.batches.size as usize);

    for _ in 0..header.batches.size {
        batches.push(M2Batch {
            flags: read_u8(r)?,
            priority: read_u8(r)?,
            shader_id: read_u16(r)?,

            submesh_index: read_u16(r)?,
            material_index: read_u16(r)?,

            texture_index: read_u16(r)?,
            tex_unit_index: read_u16(r)?,

            transparency_index: read_u16(r)?,
            texture_anim_index: read_u16(r)?,
        });
    }

    Ok(batches)
}

pub fn parse_skin(data: &[u8]) -> Result<M2Skin> {
    let mut r = Cursor::new(data);
    let header = parse_header(&mut r)?;
    r.seek(SeekFrom::Start(header.vertices.offset as u64))?;

    let mut vertices = Vec::with_capacity(header.vertices.size as usize);
    for _ in 0..header.vertices.size {
        let mut b = [0u8; 2];
        r.read_exact(&mut b)?;
        vertices.push(u16::from_le_bytes(b));
    }
    r.seek(SeekFrom::Start(header.indices.offset as u64))?;

    let mut indices = Vec::with_capacity(header.indices.size as usize);
    for _ in 0..header.indices.size {
        let mut b = [0u8; 2];
        r.read_exact(&mut b)?;
        indices.push(u16::from_le_bytes(b));
    }

    let submeshes = read_submeshes(&header, &mut r)?;
    let batches = read_batches(&header, &mut r)?;

    Ok(M2Skin {
        vertices,
        indices,
        submeshes,
        batches,
    })
}
