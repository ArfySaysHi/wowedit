use scene::terrain::Terrain;

use crate::{
    terrain_alpha::ChunkAlphaMaps,
    terrain_mesh::{ChunkGpuBuffers, ChunkMesh},
    terrain_pipeline::TerrainPipeline,
};

pub struct TerrainRenderer {
    pipeline: TerrainPipeline,
    chunks: Vec<ChunkGpuBuffers>,
    alpha_maps: Vec<ChunkAlphaMaps>,
    alpha_layout: wgpu::BindGroupLayout,
}

impl TerrainRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let alpha_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain_texture_alpha_map_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline = TerrainPipeline::new(
            device,
            surface_format,
            camera_layout,
            texture_layout,
            &alpha_layout,
        );

        Self {
            pipeline,
            chunks: Vec::new(),
            alpha_maps: Vec::new(),
            alpha_layout,
        }
    }

    pub fn load_terrain(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, terrain: &Terrain) {
        for chunk in &terrain.chunks {
            let mesh = ChunkMesh::from_chunk(chunk);
            let gpu_buffers = ChunkGpuBuffers::upload(device, &mesh);
            let alpha_maps = ChunkAlphaMaps::new(
                device,
                queue,
                &chunk.mcal.alpha_maps,
                &self.alpha_layout,
                &chunk.layers,
            );

            self.chunks.push(gpu_buffers);
            self.alpha_maps.push(alpha_maps);
        }
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        texture_bind_group: &'a wgpu::BindGroup,
    ) {
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);
        pass.set_bind_group(1, texture_bind_group, &[]);

        for (chunk, alpha) in self.chunks.iter().zip(self.alpha_maps.iter()) {
            pass.set_bind_group(2, &alpha.bind_group, &[]);
            pass.set_vertex_buffer(0, chunk.vertex_buffer.slice(..));
            pass.set_index_buffer(chunk.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..chunk.index_count, 0, 0..1);
        }
    }
}
