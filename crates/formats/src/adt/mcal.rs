use crate::adt::mcly::Mcly;
use anyhow::{Result, bail};

const ALPHA_MAP_SIZE: usize = 64 * 64;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Mcal {
    /// One alpha map per layer after the base, in layer order.
    /// Each map is 4096 bytes (64x64), values 0-255.
    pub alpha_maps: Vec<[u8; ALPHA_MAP_SIZE]>,
}

pub fn parse(data: &[u8], layers: &Mcly) -> Result<Mcal> {
    let mut alpha_maps = Vec::new();

    // Base layer has no alpha map, start from layer 1
    for layer in layers.layers.iter().skip(1) {
        let offset = layer.offset_in_mcal as usize;

        let map = if layer.flags & 0x200 != 0 {
            // Uncompressed: 4096 bytes, one u8 per texel
            if offset + ALPHA_MAP_SIZE > data.len() {
                bail!("MCAL uncompressed alpha map out of bounds at offset {offset}");
            }
            let mut map = [0u8; ALPHA_MAP_SIZE];
            map.copy_from_slice(&data[offset..offset + ALPHA_MAP_SIZE]);
            map
        } else if layer.flags & 0x100 != 0 {
            // 4-bit packed: 2048 bytes, two texels per byte, low nibble first
            if offset + ALPHA_MAP_SIZE / 2 > data.len() {
                bail!("MCAL 4-bit packed alpha map out of bounds at offset {offset}");
            }
            let mut map = [0u8; ALPHA_MAP_SIZE];
            for (i, byte) in data[offset..offset + ALPHA_MAP_SIZE / 2].iter().enumerate() {
                map[i * 2] = (byte & 0x0F) << 4 | (byte & 0x0F);
                map[i * 2 + 1] = (byte & 0xF0) | (byte >> 4);
            }
            map
        } else {
            // RLE compressed
            decompress(&data[offset..], ALPHA_MAP_SIZE)?
        };

        alpha_maps.push(map);
    }

    Ok(Mcal { alpha_maps })
}

/// RLE decompression for MCAL compressed alpha maps.
///
/// Each byte is a control byte:
/// - High bit set (0x80)  → fill mode:  repeat the next byte for (count) iterations
/// - High bit clear       → copy mode:  copy the next (count) bytes literally
fn decompress(data: &[u8], output_size: usize) -> Result<[u8; ALPHA_MAP_SIZE]> {
    let mut out = [0u8; ALPHA_MAP_SIZE];
    let mut i = 0; // read position in data
    let mut j = 0; // write position in out

    while j < output_size {
        if i >= data.len() {
            bail!("MCAL compressed data ended early at output byte {j}");
        }

        let control = data[i];
        i += 1;

        let fill = control & 0x80 != 0;
        let count = (control & 0x7F) as usize;

        if fill {
            if i >= data.len() {
                bail!("MCAL fill mode missing value byte at input byte {i}");
            }
            let value = data[i];
            i += 1;
            let end = (j + count).min(output_size);
            out[j..end].fill(value);
            j = end;
        } else {
            let end = (j + count).min(output_size);
            let copy_len = end - j;
            let src_end = i + copy_len;
            if src_end > data.len() {
                bail!(
                    "MCAL copy mode out of bounds: need {src_end} bytes, have {}",
                    data.len()
                );
            }
            out[j..end].copy_from_slice(&data[i..src_end]);
            i += copy_len;
            j = end;
        }
    }

    Ok(out)
}
