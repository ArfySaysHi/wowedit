use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, ShaderStages,
    StorageTextureAccess, Texture, TextureFormat, TextureSampleType, TextureViewDescriptor,
    TextureViewDimension, include_wgsl,
};

pub struct TerrainMipmapGenerator {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl TerrainMipmapGenerator {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(include_wgsl!("../shaders/terrain_mipmap.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mipmap_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("mipmap_pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("mipmap_compute_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("generate_mipmap"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    /// Generate mipmaps for a single layer of a texture array
    ///
    /// This processes one mipmap level at a time, reading from level N-1
    /// and writing to level N.
    pub fn generate_for_layer(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        texture: &Texture,
        layer_index: u32,
    ) {
        let width = texture.size().width;
        let height = texture.size().height;

        // Calculate number of mipmap levels
        let max_dim = width.max(height);
        let mip_count = max_dim.ilog2(); // Don't include level 0

        // Process each mipmap level
        for mip_level in 1..=mip_count {
            // Calculate dimensions for this mipmap level
            // Each level is half the size of the previous
            let dst_width = width >> mip_level;
            let dst_height = height >> mip_level;

            // Skip if destination would be 0-sized
            if dst_width == 0 || dst_height == 0 {
                continue;
            }

            // Create view for source mipmap level (previous level)
            let src_view = texture.create_view(&TextureViewDescriptor {
                label: Some(&format!(
                    "mipmap_src_layer{}_level{}",
                    layer_index,
                    mip_level - 1
                )),
                dimension: Some(TextureViewDimension::D2),
                base_mip_level: mip_level - 1,
                mip_level_count: Some(1),
                base_array_layer: layer_index,
                array_layer_count: Some(1),
                ..Default::default()
            });

            // Create view for destination mipmap level (current level)
            let dst_view = texture.create_view(&TextureViewDescriptor {
                label: Some(&format!(
                    "mipmap_dst_layer{}_level{}",
                    layer_index, mip_level
                )),
                dimension: Some(TextureViewDimension::D2),
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                base_array_layer: layer_index,
                array_layer_count: Some(1),
                ..Default::default()
            });

            // Create bind group for this mipmap pass
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some(&format!(
                    "mipmap_bind_group_layer{}_level{}",
                    layer_index, mip_level
                )),
                layout: &self.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                ],
            });

            // Dispatch compute shader
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some(&format!(
                    "mipmap_pass_layer{}_level{}",
                    layer_index, mip_level
                )),
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);

            // Calculate number of work groups needed
            // We use 8x8 thread groups, so divide dimensions by 8 and round up
            let workgroup_count_x = dst_width.div_ceil(8);
            let workgroup_count_y = dst_height.div_ceil(8);

            pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }
    }
}
