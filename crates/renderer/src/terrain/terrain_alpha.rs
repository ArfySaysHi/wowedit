use formats::adt::mcly::Mcly;

/// The GPU upload for alphamaps
pub struct ChunkAlphaMaps {
    pub bind_group: wgpu::BindGroup,

    // Stops the buffer being dropped early (bind_group holds an
    // internal reference but it's a little fragile, don't want it biting me later)
    _material_buffer: wgpu::Buffer,
}

/// CPU-side data definition
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ChunkMaterial {
    layer_count: u32,
    _pad: [u32; 3], // uniforms must be 16-byte aligned so... pad I guess
    texture_ids: [u32; 4],
}

impl ChunkAlphaMaps {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        alpha_maps: &[[u8; 4096]],
        alpha_layout: &wgpu::BindGroupLayout,
        layers: &Mcly,
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 3, // texture has 3 layers (first texture has no alpha)
        };

        let layer_size = wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        };

        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terrain_texture_alpha_map"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Only uses R but this means it can go through the mipmap compute shader pipeline alongside the diffuse texture
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let chunk_mat_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("chunk_mat_buffer"),
            size: std::mem::size_of::<ChunkMaterial>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        for (index, map) in alpha_maps.iter().enumerate() {
            // 1. Expand R -> RGBA
            // The input 'map' is [u8; 4096] (R only). We need [u8; 16384] (RGBA).
            let mut rgba_data = Vec::with_capacity(4096 * 4);
            for &alpha in map.iter() {
                // Replicate the alpha value into R, G, B, A channels
                // This allows the mipmap shader to treat it exactly like a color texture
                rgba_data.extend_from_slice(&[alpha, alpha, alpha, alpha]);
            }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture_array,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: index as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba_data, // 2. Pass the expanded buffer
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(64 * 4), // 4 bytes per pixel now
                    rows_per_image: Some(64),
                },
                layer_size,
            );
        }

        let view = texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let mut texture_ids = [0u32; 4];
        for (i, layer) in layers.layers.iter().enumerate().take(4) {
            texture_ids[i] = layer.texture_id;
        }

        let chunk_material = ChunkMaterial {
            layer_count: alpha_maps.len() as u32 + 1,
            _pad: [0; 3],
            texture_ids,
        };
        queue.write_buffer(&chunk_mat_buffer, 0, bytemuck::bytes_of(&chunk_material));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain_texture_alpha_map_bind_group"),
            layout: alpha_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: chunk_mat_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            bind_group,
            _material_buffer: chunk_mat_buffer,
        }
    }
}
