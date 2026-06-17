pub mod mcal;
pub mod mcly;
pub mod mcnk;
pub mod mddf;
pub mod mtex;

use crate::{adt::mddf::MddfEntry, chunks::ChunkHeader, io::read_nullterm_strings_at};
use anyhow::{Result, bail};
use std::io::{Cursor, Read, Seek, SeekFrom};

pub use mcnk::Mcnk;

pub struct Adt {
    pub chunks: Vec<Mcnk>,
    pub texture_paths: Vec<String>,
    pub doodad_filenames: Option<Vec<String>>,
    pub doodad_placements: Option<Vec<MddfEntry>>,
    pub tile_x: u8,
    pub tile_y: u8,
}

pub fn parse(data: &[u8]) -> Result<Adt> {
    let mut r = Cursor::new(data);
    let mut chunks = Vec::with_capacity(256);
    let mut texture_paths = Vec::new();
    let mut raw_mmdx: Option<Vec<String>> = Some(Vec::new());
    let mut raw_mddf: Option<Vec<MddfEntry>> = Some(Vec::new());

    loop {
        let header = match ChunkHeader::read(&mut r) {
            Ok(h) => h,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };

        let start = r.stream_position()?;

        match &header.magic {
            b"REVM" => { /* MVER skip */ }

            b"RDHM" => { /* MHDR skip */ }

            b"KNCM" => {
                let mut chunk_data = vec![0u8; header.size as usize];
                r.read_exact(&mut chunk_data)?;
                chunks.push(mcnk::parse(&chunk_data)?);
                continue;
            }

            b"XETM" => {
                let mut chunk_data = vec![0u8; header.size as usize];
                r.read_exact(&mut chunk_data)?;
                texture_paths = mtex::parse(&chunk_data)?.filenames;
                continue;
            }

            b"XDMM" => {
                let mut data = vec![0u8; header.size as usize];
                r.read_exact(&mut data)?;
                let mut cursor = Cursor::new(data.as_slice());
                raw_mmdx = Some(read_nullterm_strings_at(&mut cursor, 0)?);
                continue;
            }

            b"FDDM" => {
                let mut data = vec![0u8; header.size as usize];
                r.read_exact(&mut data)?;
                raw_mddf = Some(mddf::parse(&data)?);
                println!("{:?}", raw_mddf.as_ref().unwrap().first());
                continue;
            }

            _ => { /* unknown chunk skip */ }
        }

        r.seek(SeekFrom::Start(start + header.size as u64))?;
    }

    if chunks.len() != 256 {
        bail!("expected 256 MCNK chunks, got {}", chunks.len());
    }

    Ok(Adt {
        chunks,
        texture_paths,
        doodad_filenames: raw_mmdx,
        doodad_placements: raw_mddf,
        tile_x: 0,
        tile_y: 0,
    })
}
