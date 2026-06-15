use anyhow::Result;
use wow_blp::{convert::blp_to_image, parser::load_blp_from_buf};

pub struct BlpImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Loads and decodes a BLP image from bytes to RGBA
pub fn decode(data: &[u8]) -> Result<BlpImage> {
    let blp = load_blp_from_buf(data)?;
    let image = blp_to_image(&blp, 0)?;
    let rgba = image.to_rgba8().into_raw();
    let height = image.height();
    let width = image.width();

    Ok(BlpImage {
        rgba,
        width,
        height,
    })
}
