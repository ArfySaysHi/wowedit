use crate::{
    adt::{
        mcal::{self, Mcal},
        mcly::{self, Mcly},
    },
    chunks::ChunkHeader,
    io::{read_f32_at, read_u32_at},
};
use anyhow::Result;
use glam::Vec3;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Mcnk {
    pub position: Vec3,
    pub heights: [f32; 145],
    pub area_id: u32,
    pub holes: u32,
    pub layers: Mcly,
    pub mcal: Mcal,
}

pub fn parse(data: &[u8]) -> Result<Mcnk> {
    let mut r = Cursor::new(data);

    let area_id = read_u32_at(&mut r, 52)?;
    let holes = read_u32_at(&mut r, 60)?;
    let pos_x = read_f32_at(&mut r, 104)?;
    let pos_y = read_f32_at(&mut r, 108)?;
    let pos_z = read_f32_at(&mut r, 112)?;

    let mut heights: Option<[f32; 145]> = None;
    let mut layers: Option<Mcly> = None;
    let mut raw_mcal: Option<Vec<u8>> = None;

    // Subchunks begin after the 128-byte MCNK header
    r.seek(SeekFrom::Start(128))?;

    let end = data.len() as u64;

    while r.position() < end {
        let header = ChunkHeader::read(&mut r)?;
        let next = r.position() + header.size as u64;

        match &header.magic {
            b"TVCM" => {
                let mut buf = [0f32; 145];
                for h in buf.iter_mut() {
                    let mut bytes = [0u8; 4];
                    r.read_exact(&mut bytes)?;
                    *h = f32::from_le_bytes(bytes);
                }
                heights = Some(buf);
            }
            b"YLCM" => {
                let mut buffer = vec![0u8; header.size as usize];
                r.read_exact(&mut buffer)?;
                layers = Some(mcly::parse(&buffer)?);
            }
            b"RNCM" => {
                // junk bytes messing with cursor alignment, they are seemingly in all WotLK MCNR
                // chunks and are the same each time. I have no idea why they exist.
                r.seek(SeekFrom::Start(next + 13))?;
                continue;
            }
            b"LACM" => {
                let mut buf = vec![0u8; header.size as usize];
                r.read_exact(&mut buf)?;
                raw_mcal = Some(buf);
            }
            _ => {}
        }

        r.seek(SeekFrom::Start(next))?;
    }

    let layers = layers.ok_or_else(|| anyhow::anyhow!("missing MCLY"))?;
    let mcal = if let Some(raw) = raw_mcal {
        mcal::parse(&raw, &layers)?
    } else {
        Mcal::default() // chunks with only a base layer may have no MCAL
    };

    Ok(Mcnk {
        position: Vec3::new(pos_x, pos_y, pos_z),
        heights: heights.ok_or_else(|| anyhow::anyhow!("MCNK missing MCVT subchunk"))?,
        area_id,
        holes,
        layers,
        mcal,
    })
}
