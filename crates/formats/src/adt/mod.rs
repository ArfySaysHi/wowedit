mod mcnk;
mod mhdr;

use crate::chunks::ChunkHeader;
use anyhow::{Result, bail};
use std::io::{Cursor, Read, Seek, SeekFrom};

pub use mcnk::Mcnk;

pub struct Adt {
    pub chunks: Vec<Mcnk>,
}

pub fn parse(data: &[u8]) -> Result<Adt> {
    let mut r = Cursor::new(data);
    let mut chunks = Vec::with_capacity(256);

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
                let chunk_data = read_chunk_data(&mut r, header.size)?;
                chunks.push(mcnk::parse(&chunk_data)?);
                continue;
            }
            _ => { /* unknown chunk skip */ }
        }

        r.seek(SeekFrom::Start(start + header.size as u64))?;
    }

    if chunks.len() != 256 {
        bail!("expected 256 MCNK chunks, got {}", chunks.len());
    }

    Ok(Adt { chunks })
}

fn read_chunk_data(r: &mut Cursor<&[u8]>, size: u32) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; size as usize];
    r.read_exact(&mut buf)?;
    Ok(buf)
}
