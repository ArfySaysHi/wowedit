use crate::m2::{
    m2_instance_buffer::M2InstanceBuffer, m2_mesh::M2GpuMesh, m2_pipeline::M2Pipeline,
};
use formats::{blp::BlpImage, m2::m2_model::M2Model};
use glam::Mat4;
use std::{collections::HashMap, sync::Arc};

pub struct M2GpuBatch {
    pub start_index: u32,
    pub index_count: u32,
    pub texture_index: usize,
}

// One entry per unique M2 model loaded
struct M2DrawCall {
    mesh: M2GpuMesh,
    instances: M2InstanceBuffer,

    textures: Vec<Option<Arc<wgpu::BindGroup>>>,
}

pub struct M2Renderer {
    pipeline: M2Pipeline,
    draw_calls: Vec<M2DrawCall>,
    texture_cache: HashMap<String, Arc<wgpu::BindGroup>>,
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
            texture_cache: HashMap::new(),
        }
    }

    pub fn load(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model: &M2Model,
        transforms: &[Mat4],
        blp_images: &[Option<BlpImage>],
    ) {
        let texture_bind_groups = model
            .texture_paths
            .iter()
            .zip(blp_images.iter())
            .map(|(path, image)| {
                image.as_ref().map(|image| {
                    let bg = self.texture_cache.entry(path.clone()).or_insert_with(|| {
                        Arc::new(upload_texture(
                            device,
                            queue,
                            image,
                            &self.pipeline.texture_layout,
                        ))
                    });

                    Arc::clone(bg)
                })
            })
            .collect::<Vec<Option<Arc<wgpu::BindGroup>>>>();

        let mesh = M2GpuMesh::upload(device, model);
        let matrices: Vec<[f32; 16]> = transforms.iter().map(|m| m.to_cols_array()).collect();
        let instances = M2InstanceBuffer::upload(device, &matrices);

        self.draw_calls.push(M2DrawCall {
            mesh,
            instances,
            textures: texture_bind_groups,
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
            let texture = call.textures.first().unwrap().as_ref().unwrap();

            pass.set_bind_group(1, texture.as_ref(), &[]);
            pass.set_vertex_buffer(0, call.mesh.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, call.instances.buffer.slice(..));
            pass.set_index_buffer(call.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..call.mesh.index_count, 0, 0..call.instances.count);
        }
    }
}

fn upload_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &BlpImage,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("m2_texture"),
        size: wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &image.rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * image.width),
            rows_per_image: Some(image.height),
        },
        wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        },
    );

    let view = texture.create_view(&Default::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("m2_sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("m2_texture_bind_group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}
