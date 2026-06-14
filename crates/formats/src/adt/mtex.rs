use crate::io::read_nullterm_strings_at;
use anyhow::Result;
use std::io::Cursor;

pub struct Mtex {
    pub filenames: Vec<String>,
}

pub fn parse(data: &[u8]) -> Result<Mtex> {
    let mut r = Cursor::new(data);
    let filenames = read_nullterm_strings_at(&mut r, 0)?;

    Ok(Mtex { filenames })
}
