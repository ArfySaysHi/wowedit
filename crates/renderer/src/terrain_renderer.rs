use crate::{
    terrain_mesh::{ChunkGpuBuffers, ChunkMesh},
    terrain_pipeline::TerrainPipeline,
};

pub struct TerrainRenderer {
    pipeline: TerrainPipeline,
    chunks: Vec<ChunkGpuBuffers>,
}

impl TerrainRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        terrain: &scene::terrain::Terrain,
        camera_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let pipeline = TerrainPipeline::new(device, surface_format, camera_layout);

        let chunks = terrain
            .chunks
            .iter()
            .map(|c| {
                let mesh = ChunkMesh::from_chunk(c);
                println!(
                    "vertices: {}, indices: {}",
                    mesh.vertices.len(),
                    mesh.indices.len()
                );
                ChunkGpuBuffers::upload(device, &mesh)
            })
            .collect::<Vec<ChunkGpuBuffers>>();

        Self { pipeline, chunks }
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);

        for chunk in &self.chunks {
            // pass.set_vertex_buffer(0, chunk.vertex_buffer.slice(..));
            // pass.set_index_buffer(chunk.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // pass.draw_indexed(0..chunk.index_count, 0, 0..1);
            pass.draw(0..3, 0..1);
        }
    }
}
