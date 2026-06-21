use anyhow::Result;
use formats::loader::AssetLoader;
use std::{collections::HashMap, sync::Arc};

/// A single GPU texture plus its own one-texture bind group.
/// Each M2 batch/submesh holds an index into `TextureManager::textures`
/// and binds this group directly before drawing — no texture arrays,
/// no bindless features/limits required.
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

pub struct TextureManager {
    textures: Vec<Arc<GpuTexture>>,
    lookup: HashMap<String, u32>,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}

impl TextureManager {
    pub fn new(device: &wgpu::Device) -> Self {
        // One texture + one sampler per bind group. No `count` on the
        // texture entry == not a binding array, so no extra features
        // or limits are needed on the device.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_manager_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
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
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_manager_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        Self {
            textures: Vec::new(),
            lookup: HashMap::new(),
            bind_group_layout,
            sampler,
        }
    }

    /// Loads a texture by path if not already loaded, returning a stable
    /// index that M2Batch/M2Submesh can store and resolve later via `get`.
    pub fn get_or_load(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        loader: &AssetLoader,
        path: &str,
    ) -> Result<u32> {
        if let Some(index) = self.lookup.get(path) {
            return Ok(*index);
        }

        let blp = loader.load_blp(path)?;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(path),
            size: wgpu::Extent3d {
                width: blp.width,
                height: blp.height,
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
            &blp.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(blp.width * 4),
                rows_per_image: Some(blp.height),
            },
            wgpu::Extent3d {
                width: blp.width,
                height: blp.height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(path),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(path),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let index = self.textures.len() as u32;

        self.textures.push(Arc::new(GpuTexture {
            texture,
            view,
            bind_group,
        }));

        self.lookup.insert(path.to_owned(), index);

        Ok(index)
    }

    /// Resolves a texture index (as returned by `get_or_load`, and as
    /// stored per-submesh/per-batch) to its GPU texture + bind group.
    pub fn get(&self, index: u32) -> Option<&Arc<GpuTexture>> {
        self.textures.get(index as usize)
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        panic!("TextureManager requires a Device")
    }
}
