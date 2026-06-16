use crate::terrain::terrain_mipmap::TerrainMipmapGenerator;
use formats::blp::BlpImage;
use std::collections::HashMap;
use wgpu::{CommandEncoderDescriptor, Texture};

pub struct TerrainTextures {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub texture_array: Texture,
}

impl TerrainTextures {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        textures: &HashMap<u32, BlpImage>,
        mipmap_generator: &TerrainMipmapGenerator,
    ) -> Self {
        let layer_count = textures.len() as u32;

        let max_dimension = 256u32;
        let mip_level_count = max_dimension.ilog2() + 1;

        let texture_size = wgpu::Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: layer_count,
        };

        // Create the texture array
        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terrain_texture_array"),
            size: texture_size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        // Upload each texture into its corresponding array layer (only level 0)
        let mut indices: Vec<u32> = textures.keys().copied().collect();
        indices.sort();

        for index in indices {
            let image = &textures[&index];
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture_array,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: index,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &image.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * 256),
                    rows_per_image: Some(256),
                },
                wgpu::Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
            );
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("mipmap_generation_encoder"),
        });

        for layer_index in 0..layer_count {
            mipmap_generator.generate_for_layer(device, &mut encoder, &texture_array, layer_index);
        }

        queue.submit(Some(encoder.finish()));

        let view = texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_mip_level: 0,
            mip_level_count: Some(mip_level_count),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: mip_level_count as f32,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain_textures_bind_group_layout"),
            entries: &[
                // Texture array
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
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain_textures_bind_group"),
            layout: &bind_group_layout,
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
        });

        Self {
            bind_group,
            bind_group_layout,
            texture_array,
        }
    }
}
