use anyhow::Result;
use std::io::{BufRead, Cursor, Read, Seek, SeekFrom};

pub fn read_u32_at(r: &mut Cursor<&[u8]>, offset: u64) -> Result<u32> {
    r.seek(SeekFrom::Start(offset))?;
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_f32_at(r: &mut Cursor<&[u8]>, offset: u64) -> Result<f32> {
    r.seek(SeekFrom::Start(offset))?;
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

pub fn read_u32(r: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_f32(r: &mut Cursor<&[u8]>) -> Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

pub fn read_nullterm_strings_at(r: &mut Cursor<&[u8]>, offset: u64) -> Result<Vec<String>> {
    r.seek(SeekFrom::Start(offset))?;

    let mut strings = Vec::new();

    loop {
        let mut buf = Vec::new();
        r.read_until(0, &mut buf)?;

        if buf.is_empty() {
            break;
        }

        if buf.last() == Some(&0) {
            buf.pop();
        }

        if buf.is_empty() {
            break;
        }

        strings.push(String::from_utf8_lossy(&buf).into_owned());
    }

    Ok(strings)
}
