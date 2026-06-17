use wgpu::util::DeviceExt;

pub struct M2InstanceBuffer {
    pub buffer: wgpu::Buffer,
    pub count: u32,
}

impl M2InstanceBuffer {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: 64, // size of one mat4x4
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn upload(device: &wgpu::Device, matrices: &[[f32; 16]]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("m2_instance_buffer"),
            contents: bytemuck::cast_slice(matrices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            buffer,
            count: matrices.len() as u32,
        }
    }
}
