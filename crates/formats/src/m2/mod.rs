use crate::{
    io::{read_f32, read_u16, read_u32, read_u32_at},
    m2::{m2_texture::M2Texture, m2_vertex::M2Vertex},
};
use anyhow::{Result, bail};
use std::io::{Cursor, Read, Seek, SeekFrom};

pub mod m2_model;
pub mod m2_resolved_mesh;
pub mod m2_skin;
pub mod m2_texture;
pub mod m2_vertex;

// We take only what we need to render for now
#[derive(Debug)]
pub struct M2Header {
    // magic: [u8; 4]  — always b"MD20", read and validate but don't store
    // version: u32    — should be 264 for WotLK
    pub vertices_count: u32,
    pub vertices_offset: u32,

    pub textures_count: u32,
    pub textures_offset: u32,

    pub materials_count: u32,
    pub materials_offset: u32,

    pub texture_lookup_count: u32,
    pub texture_lookup_offset: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct M2Material {
    pub flags: u16,
    pub blend_mode: u16,
}

pub fn parse_render_flags(data: &[u8], offset: usize, count: usize) -> Result<Vec<M2Material>> {
    let mut r = Cursor::new(data);
    r.seek(SeekFrom::Start(offset as u64))?;
    let mut materials = Vec::with_capacity(count);
    for _ in 0..count {
        materials.push(M2Material {
            flags: read_u16(&mut r)?,
            blend_mode: read_u16(&mut r)?,
        });
    }
    Ok(materials)
}

pub fn parse_header(data: &[u8]) -> Result<M2Header> {
    let mut r = Cursor::new(data);

    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != b"MD20" {
        bail!("Not an M2 file");
    }

    Ok(M2Header {
        vertices_count: read_u32_at(&mut r, 0x3C)?,
        vertices_offset: read_u32_at(&mut r, 0x40)?,
        textures_count: read_u32_at(&mut r, 0x50)?,
        textures_offset: read_u32_at(&mut r, 0x54)?,
        materials_count: read_u32_at(&mut r, 0x70)?,
        materials_offset: read_u32_at(&mut r, 0x74)?,
        texture_lookup_count: read_u32_at(&mut r, 0x80)?,
        texture_lookup_offset: read_u32_at(&mut r, 0x84)?,
    })
}

pub fn parse_vertices(data: &[u8], offset: usize, count: usize) -> Result<Vec<M2Vertex>> {
    let mut vertices = Vec::with_capacity(count);
    let mut r = Cursor::new(data);
    r.seek(SeekFrom::Start(offset as u64))?;

    for _ in 0..count {
        vertices.push(M2Vertex {
            position: [read_f32(&mut r)?, read_f32(&mut r)?, read_f32(&mut r)?],
            bone_weights: {
                let mut b = [0u8; 4];
                r.read_exact(&mut b)?;
                b
            },
            bone_indices: {
                let mut b = [0u8; 4];
                r.read_exact(&mut b)?;
                b
            },
            normal: [read_f32(&mut r)?, read_f32(&mut r)?, read_f32(&mut r)?],
            tex_coords: [
                [read_f32(&mut r)?, read_f32(&mut r)?],
                [read_f32(&mut r)?, read_f32(&mut r)?],
            ],
        });
    }

    Ok(vertices)
}

pub fn parse_textures(data: &[u8], offset: usize, count: usize) -> Result<Vec<M2Texture>> {
    let mut textures = Vec::with_capacity(count);
    let mut r = Cursor::new(data);
    r.seek(SeekFrom::Start(offset as u64))?;

    for _ in 0..count {
        textures.push(M2Texture {
            texture_type: read_u32(&mut r)?,
            flags: read_u32(&mut r)?,
            filename_length: read_u32(&mut r)?,
            filename_offset: read_u32(&mut r)?,
        });
    }

    Ok(textures)
}

pub fn parse_texture_lookup(data: &[u8], offset: usize, count: usize) -> Result<Vec<u16>> {
    let mut r = Cursor::new(data);
    r.seek(SeekFrom::Start(offset as u64))?;
    let mut table = Vec::with_capacity(count);
    for _ in 0..count {
        let mut b = [0u8; 2];
        r.read_exact(&mut b)?;
        table.push(u16::from_le_bytes(b));
    }
    Ok(table)
}
