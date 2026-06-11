use crate::vertex::Vertex;
use scene::terrain::TerrainChunk;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const CHUNK_SIZE: f32 = 533.333_3; // The entire ADT size in units
const CELL_SIZE: f32 = CHUNK_SIZE / 8.0; // 66.6666666...

pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    const CHUNK_VERTICES: usize = 145;
    const CHUNK_INDICES: usize = 768;

    pub fn from_chunk(chunk: &TerrainChunk) -> Self {
        let origin = chunk.world_position;
        let mut vertices: Vec<Vertex> = Vec::with_capacity(Self::CHUNK_VERTICES);
        let mut indices = Vec::with_capacity(Self::CHUNK_INDICES);

        for idx in 0..Self::CHUNK_VERTICES {
            let (offset_x, offset_y) = vertex_local_pos(idx);
            vertices.push(Vertex {
                position: [
                    origin.x + offset_x,
                    origin.y + chunk.heights[idx],
                    origin.z + offset_y,
                ],
            });
        }

        for row in 0..8 {
            for col in 0..8 {
                indices.extend_from_slice(&cell_indices(row, col));
            }
        }

        Self { vertices, indices }
    }
}

pub struct ChunkGpuBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl ChunkGpuBuffers {
    pub fn upload(device: &wgpu::Device, mesh: &ChunkMesh) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("chunk_vb"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("chunk_ib"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        }
    }
}

// Gets the position of the vertex within a flat vector of values with the knowledge
// that the first 9 are 'outer' vertices and the last 8 are 'inner' vertices (sum 17)
fn vertex_local_pos(idx: usize) -> (f32, f32) {
    // Each row has 17 vertices so divide it by 17 to get the row it's on
    // Example: If the index is 35, 35/17 is 2 remainder 1 | 2 is our row and 1 is our column
    let row = idx / 17;
    let col = idx % 17;

    if col < 9 {
        // Is an outer vertex
        let r = row;
        let c = col;
        (r as f32 * CELL_SIZE, c as f32 * CELL_SIZE)
    } else {
        // Is an inner vertex
        let r = row;
        let c = col - 9;

        // Multiply CELL_SIZE by 0.5 to place the inner vertex halfway between
        // the outer vertices, allows the cell to be divided (see cell_indices below)
        (
            r as f32 * CELL_SIZE + CELL_SIZE * 0.5,
            c as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        )
    }
}

// Creates an index buffer for the four triangles that make up a cell, implicitly required
// that the provided coordinates point to an inner vertex
fn cell_indices(row: usize, col: usize) -> [u32; 12] {
    let tl = (row * 17 + col) as u32; // topleft
    let bl = ((row + 1) * 17 + col) as u32; // bottomleft
    let br = ((row + 1) * 17 + col + 1) as u32; // bottomright
    let tr = (row * 17 + col + 1) as u32; // topright
    let cr = (row * 17 + 9 + col) as u32; // center

    // The cell is split into 4 triangles around the center vertex
    // tl ----- tr
    // | \    / |
    // |   cr   |
    // | /    \ |
    // bl ----- br

    #[rustfmt::skip]
    let indices = [
        tl, cr, tr,
        tr, cr, br,
        br, cr, bl,
        bl, cr, tl
    ];

    // 12 indices per cell, there are 64 cells per chunk so a total of 768 indices
    indices
}
