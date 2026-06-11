use crate::wgpu_state::WgpuState;
use formats::{loader::AssetLoader, storage::CompoundStorage, version::WoWVersion};
use glam::Vec3;
use renderer::{gpu_camera::GpuCamera, terrain_mesh::ChunkMesh, terrain_renderer::TerrainRenderer};
use scene::camera::Camera;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

// Owns everything that lasts until the application is closed
#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    wgpu: Option<WgpuState>,
    terrain_renderer: Option<TerrainRenderer>,
    depth_texture: Option<wgpu::TextureView>,
    gpu_camera: Option<GpuCamera>,
    camera: Option<Camera>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("wowedit"))
                .unwrap(),
        );

        let wgpu = pollster::block_on(WgpuState::new(Arc::clone(&window))).unwrap();

        // Load camera
        let camera = Camera::new(Vec3::new(-4000.0, 9500.0, -4000.0))
            .with_pitch(-std::f32::consts::FRAC_PI_3);
        let gpu_camera = GpuCamera::new(&wgpu.device);

        // Load terrain
        let storage = CompoundStorage::from_wow_install(
            "/home/arfy/Games/acwow/ChromieCraft_3.3.5a/Data",
            "enUS",
        )
        .unwrap();
        let loader = AssetLoader::new(Box::new(storage), WoWVersion::WotLK);
        let adt = loader.load_adt("Azeroth", 32, 48).unwrap();
        let terrain = scene::terrain::Terrain::from(adt);

        for chunk in &terrain.chunks[..4] {
            let mesh = ChunkMesh::from_chunk(chunk);
            println!(
                "Chunk at {:?}, first 3 verts: {:?}",
                chunk.world_position,
                &mesh.vertices[0..3]
            );
        }

        let terrain_renderer = TerrainRenderer::new(
            &wgpu.device,
            wgpu.surface_config.format,
            &terrain,
            &gpu_camera.bind_group_layout,
        );

        let depth_texture = create_depth_texture(&wgpu.device, &wgpu.surface_config);

        self.camera = Some(camera);
        self.gpu_camera = Some(gpu_camera);
        self.depth_texture = Some(depth_texture);
        self.terrain_renderer = Some(terrain_renderer);
        self.window = Some(window);
        self.wgpu = Some(wgpu);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(wgpu) = &mut self.wgpu {
                    wgpu.resize(size);

                    // Depth texture stops z-fighting after resize
                    let depth_texture = create_depth_texture(&wgpu.device, &wgpu.surface_config);
                    self.depth_texture = Some(depth_texture);
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }

                if let (
                    Some(wgpu),
                    Some(terrain_renderer),
                    Some(depth_texture),
                    Some(gpu_camera),
                    Some(camera),
                ) = (
                    &mut self.wgpu,
                    &self.terrain_renderer,
                    &self.depth_texture,
                    &self.gpu_camera,
                    &self.camera,
                ) {
                    // Make sure our camera is in position
                    let aspect =
                        wgpu.surface_config.width as f32 / wgpu.surface_config.height as f32;
                    let view_proj = camera.build_view_proj(aspect);
                    gpu_camera.update_camera(&wgpu.queue, &view_proj);

                    // Render the frame
                    match wgpu.render(terrain_renderer, depth_texture, gpu_camera) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{e}");
                            event_loop.exit();
                        }
                    }
                }
            }

            _ => {}
        }
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    texture.create_view(&Default::default())
}
