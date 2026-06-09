use std::io::{self, Read};

pub struct ChunkHeader {
    pub magic: [u8; 4],
    pub size: u32,
}

impl ChunkHeader {
    pub fn read(r: &mut impl Read) -> io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        let mut size_buf = [0u8; 4];
        r.read_exact(&mut size_buf)?;
        let size = u32::from_le_bytes(size_buf);
        Ok(Self { magic, size })
    }
}
