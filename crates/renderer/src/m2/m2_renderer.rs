use formats::m2::m2_model::M2Model;
use glam::Mat4;

use crate::m2::{
    m2_instance_buffer::M2InstanceBuffer, m2_mesh::M2GpuMesh, m2_pipeline::M2Pipeline,
};

// One entry per unique M2 model loaded
struct M2DrawCall {
    mesh: M2GpuMesh,
    instances: M2InstanceBuffer,
}

pub struct M2Renderer {
    pipeline: M2Pipeline,
    draw_calls: Vec<M2DrawCall>,
}

impl M2Renderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let pipeline = M2Pipeline::new(device, surface_format, camera_layout);

        Self {
            pipeline,
            draw_calls: Vec::new(),
        }
    }

    pub fn load(&mut self, device: &wgpu::Device, model: &M2Model, transforms: &[Mat4]) {
        let mesh = M2GpuMesh::upload(device, model);
        let matrices: Vec<[f32; 16]> = transforms.iter().map(|m| m.to_cols_array()).collect();
        let instances = M2InstanceBuffer::upload(device, &matrices);

        self.draw_calls.push(M2DrawCall { mesh, instances });
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);

        for call in &self.draw_calls {
            pass.set_vertex_buffer(0, call.mesh.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, call.instances.buffer.slice(..));
            pass.set_index_buffer(call.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..call.mesh.index_count, 0, 0..call.instances.count);
        }
    }
}
