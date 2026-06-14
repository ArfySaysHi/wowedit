use anyhow::Result;
use std::io::Cursor;

use crate::io::read_u32;

/// Per-chunk layer definitions for textures
#[derive(Clone, Debug)]
pub struct MclyEntry {
    pub texture_id: u32,
    pub flags: u32,
    pub offset_in_mcal: u32,
    pub effect_id: u32,
}

/// The four textures for the chunk in WotLK
#[derive(Debug, Clone)]
pub struct Mcly {
    pub layers: Vec<MclyEntry>,
}

pub fn parse(data: &[u8]) -> Result<Mcly> {
    let num_layers = data.len() / 16;
    let mut layers: Vec<MclyEntry> = Vec::new();

    let mut r = Cursor::new(data);

    for _ in 0..num_layers {
        let texture_id = read_u32(&mut r)?;
        let flags = read_u32(&mut r)?;
        let offset_in_mcal = read_u32(&mut r)?;
        let effect_id = read_u32(&mut r)?;

        layers.push(MclyEntry {
            texture_id,
            flags,
            offset_in_mcal,
            effect_id,
        });
    }

    Ok(Mcly { layers })
}
