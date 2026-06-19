use crate::m2::m2_vertex::M2GpuVertex;
use formats::m2::m2_resolved_mesh::M2ResolvedMesh;
use wgpu::util::DeviceExt;

pub struct M2GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl M2GpuMesh {
    pub fn upload(device: &wgpu::Device, model: &M2ResolvedMesh) -> Self {
        let gpu_vertices: Vec<M2GpuVertex> = model
            .vertices
            .iter()
            .map(|v| M2GpuVertex::from(*v))
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("m2_vertex_buffer"),
            contents: bytemuck::cast_slice(&gpu_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("m2_index_buffer"),
            contents: bytemuck::cast_slice(&model.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: model.indices.len() as u32,
        }
    }
}
