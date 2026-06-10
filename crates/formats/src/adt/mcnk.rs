use crate::chunks::ChunkHeader;
use anyhow::{Result, bail};
use glam::Vec3;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Mcnk {
    pub position: Vec3,
    pub heights: [f32; 145],
    pub area_id: u32,
    pub holes: u32,
}

pub fn parse(data: &[u8]) -> Result<Mcnk> {
    let mut r = Cursor::new(data);

    let area_id = read_u32_at(&mut r, 52)?;
    let holes = read_u32_at(&mut r, 60)?;
    let pos_x = read_f32_at(&mut r, 116)?;
    let pos_y = read_f32_at(&mut r, 120)?;
    let pos_z = read_f32_at(&mut r, 124)?;

    r.seek(SeekFrom::Start(128))?;
    let heights = read_mcvt(&mut r)?;

    Ok(Mcnk {
        position: Vec3::new(pos_x, pos_y, pos_z),
        heights,
        area_id,
        holes,
    })
}

/// MCVT contains terrain heights
///
/// There are 145 values arranged as:
/// - 81 outer vertices
/// - 64 inner vertices
/// - 145 total vertices
///
/// There are 9 outer vertices and 8 inner vertices on each "row":
///     O O O O O O O O O I I I I I I I I
///
fn read_mcvt(r: &mut Cursor<&[u8]>) -> Result<[f32; 145]> {
    loop {
        let header = match ChunkHeader::read(r) {
            Ok(h) => h,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                bail!("MCVT subchunk not found in MCNK")
            }
            Err(e) => return Err(e.into()),
        };

        if header.matches(b"MCVT") {
            let mut heights = [0f32; 145];
            for h in heights.iter_mut() {
                let mut buf = [0u8; 4];
                r.read_exact(&mut buf)?;
                *h = f32::from_le_bytes(buf);
            }
            return Ok(heights);
        }

        let pos = r.stream_position()?;
        r.seek(SeekFrom::Start(pos + header.size as u64))?;
    }
}

fn read_u32_at(r: &mut Cursor<&[u8]>, offset: u64) -> Result<u32> {
    r.seek(SeekFrom::Start(offset))?;
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_f32_at(r: &mut Cursor<&[u8]>, offset: u64) -> Result<f32> {
    r.seek(SeekFrom::Start(offset))?;
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}
