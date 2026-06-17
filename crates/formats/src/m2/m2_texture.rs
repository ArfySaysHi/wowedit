use crate::io::read_nullterm_string;
use anyhow::Result;
use std::io::{Cursor, Seek};

#[derive(Debug)]
pub struct M2Texture {
    pub texture_type: u32, // 0 = filename is in the file; other values = runtime lookup
    pub flags: u32,
    pub filename_length: u32,
    pub filename_offset: u32, // byte offset of the null-terminated BLP filename
}

/// Really thin wrapper around read_nullterm_string, should probably be refactored into
/// read_nullterm_string_at
pub fn get_texture_path(data: &[u8], offset: usize) -> Result<String> {
    let mut r = Cursor::new(data);
    r.seek(std::io::SeekFrom::Start(offset as u64))?;

    read_nullterm_string(&mut r)
}
