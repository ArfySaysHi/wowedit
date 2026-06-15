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
            chunks: adt.chunks.into_iter().map(TerrainChunk::from).collect(),
        }
    }
}

// In WGPU, x is east / -west and y is up / -down while z is north / -south
// This is because WGPU makes sense. WoW being a snowflake decides to do:
// y as -east / west and z as up / -down while x is -north / south
fn wow_to_engine(v: Vec3) -> Vec3 {
    Vec3::new(-v.y, v.z, -v.x)
}

impl From<formats::adt::Mcnk> for TerrainChunk {
    fn from(mcnk: formats::adt::Mcnk) -> Self {
        Self {
            world_position: wow_to_engine(mcnk.position),
            heights: mcnk.heights,
            mcal: mcnk.mcal,
            layers: mcnk.layers,
        }
    }
}
