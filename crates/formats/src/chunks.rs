use std::io::{self, Read};

pub struct ChunkHeader {
    pub magic: [u8; 4],
    pub size: u32,
}

// A generic reader for all chunked file formats
impl ChunkHeader {
    // The identifier for chunks is reversed for... reasons
    pub fn magic_str(&self) -> &str {
        std::str::from_utf8(&self.magic).unwrap_or("????")
    }

    pub fn matches(&self, doc_name: &[u8; 4]) -> bool {
        self.magic == [doc_name[3], doc_name[2], doc_name[1], doc_name[0]]
    }

    pub fn read(r: &mut impl Read) -> io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        let mut size_buf = [0u8; 4];
        r.read_exact(&mut size_buf)?;
        let size = u32::from_le_bytes(size_buf);
        Ok(Self { magic, size })
    }
}
