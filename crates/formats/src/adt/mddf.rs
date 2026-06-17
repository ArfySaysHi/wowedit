use crate::io::{read_f32, read_u32};
use anyhow::Result;
use glam::{Mat4, Quat, Vec3};
use std::io::{Cursor, Read};

#[derive(Debug, Clone)]
pub struct MddfEntry {
    /// Index into the doodad_filenames list
    pub name_id: u32,
    /// Unique identifier — can be ignored for rendering
    pub unique_id: u32,
    pub position: [f32; 3],
    /// Rotation in degrees around each axis
    pub rotation: [f32; 3],
    /// Scale, fixed-point. Divide by 1024.0 to get a float scale factor
    pub scale: u16,
    pub flags: u16,
}

pub fn parse(data: &[u8]) -> Result<Vec<MddfEntry>> {
    let entry_size = 36;
    let count = data.len() / entry_size;
    let mut entries = Vec::with_capacity(count);
    let mut r = Cursor::new(data);

    for _ in 0..count {
        entries.push(MddfEntry {
            name_id: read_u32(&mut r)?,
            unique_id: read_u32(&mut r)?,
            position: [read_f32(&mut r)?, read_f32(&mut r)?, read_f32(&mut r)?],
            rotation: [read_f32(&mut r)?, read_f32(&mut r)?, read_f32(&mut r)?],
            scale: {
                let mut b = [0u8; 2];
                r.read_exact(&mut b)?;
                u16::from_le_bytes(b)
            },
            flags: {
                let mut b = [0u8; 2];
                r.read_exact(&mut b)?;
                u16::from_le_bytes(b)
            },
        });
    }

    Ok(entries)
}

pub fn mddf_to_model_matrix(entry: &MddfEntry) -> Mat4 {
    println!("wow pos: {:?}", entry.position);
    let scale = entry.scale as f32 / 1024.0;

    // WoW stores rotation in degrees, convert to radians
    // WoW rotation order is ZXY (applied Z first, then X, then Y)
    let rot_x = entry.rotation[0].to_radians();
    let rot_y = entry.rotation[1].to_radians();
    let rot_z = entry.rotation[2].to_radians();

    let pos = Vec3::new(
        -entry.position[2], // engine X = -wow Z (east-west)
        entry.position[1],  // engine Y = wow Y (height)
        -entry.position[0], // engine Z = -wow X (north-south)
    );

    Mat4::from_scale_rotation_translation(
        Vec3::splat(scale),
        Quat::from_euler(glam::EulerRot::ZXY, rot_z, rot_x, rot_y),
        pos,
    )
}
