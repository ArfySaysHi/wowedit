use formats::adt::{mcal::Mcal, mcly::Mcly};
use glam::Vec3;

pub struct Terrain {
    pub chunks: Vec<TerrainChunk>,
}

/// WoW terrain chunks contain 145 height samples.
///
/// Layout per row (they are stored sequentially, but we will offset I / inner vertices):
///
/// O O O O O O O O O
///  I I I I I I I I
/// O O O O O O O O O
///
/// where:
/// - O = outer vertex
/// - I = inner vertex
///
/// This produces:
/// - 9 outer rows
/// - 8 inner rows
/// - 145 vertices total
pub struct TerrainChunk {
    pub world_position: Vec3,
    pub heights: [f32; 145],
    pub mcal: Mcal,
    pub layers: Mcly,
}

impl From<formats::adt::Adt> for Terrain {
    fn from(adt: formats::adt::Adt) -> Self {
        Self {
            chunks: adt.chunks.into_iter().map(from_mcnk).collect(),
        }
    }
}

fn from_mcnk(mcnk: formats::adt::Mcnk) -> TerrainChunk {
    let wow_pos = Vec3::new(
        mcnk.position.x,
        mcnk.position.z, // z is height in MCNK too
        mcnk.position.y,
    );

    let engine_pos = Vec3::new(
        -wow_pos.z, // engine X = -wow Z
        wow_pos.y,  // engine Y = height
        -wow_pos.x, // engine Z = -wow X
    );

    TerrainChunk {
        world_position: engine_pos,
        heights: mcnk.heights,
        mcal: mcnk.mcal,
        layers: mcnk.layers,
    }
}
