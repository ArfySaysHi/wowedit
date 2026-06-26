use crate::m2::{
    m2_instance_buffer::M2InstanceBuffer, m2_mesh::M2GpuMesh, m2_pipeline::M2Pipeline,
};
use crate::texture_manager::GpuTexture;
use formats::m2::m2_resolved_mesh::M2ResolvedMesh;
use glam::Mat4;
use std::sync::Arc;

pub struct M2GpuBatch {
    pub start_index: u32,
    pub index_count: u32,
    /// Resolved texture for this batch, already looked up from
    /// TextureManager at load time (after walking texture_combo_index ->
    /// texture_lookup -> texture_paths). Each batch binds its own
    /// texture's bind group directly — no shared/bindless array.
    pub texture: Arc<GpuTexture>,
}

/// This is the mesh, I want to you render it INSTANCES amount of times
struct M2DrawCall {
    mesh: M2GpuMesh,
    instances: M2InstanceBuffer,
    batches: Vec<M2GpuBatch>,
}

pub struct M2Renderer {
    pipeline: M2Pipeline,
    draw_calls: Vec<M2DrawCall>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct M2Instance {
    /// Holds the matrix that converts the model from object to world space
    pub model: [f32; 16],
}

impl M2Renderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let pipeline = M2Pipeline::new(device, surface_format, camera_layout, texture_layout);

        Self {
            pipeline,
            draw_calls: Vec::new(),
        }
    }

    /// resolve_texture maps a batch's resolved texture index (already
    /// walked through texture_combo_index -> texture_lookup -> texture_paths
    /// by the caller) to a loaded GPU texture from TextureManager.
    pub fn load(
        &mut self,
        device: &wgpu::Device,
        model: &M2ResolvedMesh,
        transforms: &[Mat4],
        resolve_texture: impl Fn(usize) -> Option<Arc<GpuTexture>>,
    ) {
        let mesh = M2GpuMesh::upload(device, model);
        let instances: Vec<M2Instance> = transforms
            .iter()
            .map(|m| M2Instance {
                model: m.to_cols_array(),
            })
            .collect();
        let instance_buffers = M2InstanceBuffer::upload(device, &instances);

        let batches: Vec<M2GpuBatch> = model
            .batches
            .iter()
            .filter_map(|b| {
                let submesh = &model.submeshes[b.submesh_index as usize];
                let texture = match resolve_texture(b.texture_combo_index as usize) {
                    Some(t) => t,
                    None => {
                        log::warn!(
                            "Skipping M2 batch: texture_combo_index {} out of range",
                            b.texture_combo_index
                        );
                        return None;
                    }
                };

                Some(M2GpuBatch {
                    start_index: submesh.index_start as u32,
                    index_count: submesh.index_count as u32,
                    texture,
                })
            })
            .collect();

        self.draw_calls.push(M2DrawCall {
            mesh,
            instances: instance_buffers,
            batches,
        });
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

            for batch in &call.batches {
                // Per-batch texture bind group, swapped per draw call.
                pass.set_bind_group(1, &batch.texture.bind_group, &[]);

                pass.draw_indexed(
                    batch.start_index..batch.start_index + batch.index_count,
                    0,
                    0..call.instances.count,
                );
            }
        }
    }
}
