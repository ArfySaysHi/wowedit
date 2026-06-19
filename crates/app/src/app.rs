use crate::wgpu_state::WgpuState;
use formats::{
    adt::mddf::mddf_to_model_matrix, loader::AssetLoader, storage::CompoundStorage,
    version::WoWVersion,
};
use glam::{Mat4, Vec3};
use renderer::{
    gpu_camera::GpuCamera,
    m2::m2_renderer::M2Renderer,
    terrain::{
        terrain_mipmap::TerrainMipmapGenerator, terrain_renderer::TerrainRenderer,
        terrain_textures::TerrainTextures,
    },
};
use scene::camera::Camera;
use std::{collections::HashMap, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

// Owns everything that lasts until the application is closed
#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    wgpu: Option<WgpuState>,
    terrain_renderer: Option<TerrainRenderer>,
    terrain_textures: Option<TerrainTextures>,
    m2_renderer: Option<M2Renderer>,
    depth_texture: Option<wgpu::TextureView>,
    gpu_camera: Option<GpuCamera>,
    camera: Option<Camera>,
    mouse_locked: Option<bool>, // This is temporary and needs to go
    egui_ctx: egui::Context,
    egui_winit: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
}

impl App {
    // Needs massive cleanup once camera code is hidden away somewhere
    // Probably choose direction by adding key values, normalize then mult by speed
    // Will also fix the locked into a direction issue
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        let speed = 10.0;
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::KeyW, true) => {
                if let Some(camera) = &mut self.camera {
                    camera.process_keyboard(scene::camera::CameraMovement::Forward, speed, 1.0);
                }
            }
            (KeyCode::KeyA, true) => {
                if let Some(camera) = &mut self.camera {
                    camera.process_keyboard(scene::camera::CameraMovement::Left, speed, 1.0);
                }
            }
            (KeyCode::KeyS, true) => {
                if let Some(camera) = &mut self.camera {
                    camera.process_keyboard(scene::camera::CameraMovement::Backward, speed, 1.0);
                }
            }
            (KeyCode::KeyD, true) => {
                if let Some(camera) = &mut self.camera {
                    camera.process_keyboard(scene::camera::CameraMovement::Right, speed, 1.0);
                }
            }
            _ => {}
        }
    }

    fn set_cursor_locked(&mut self, locked: bool) {
        if let Some(window) = &self.window {
            self.mouse_locked = Some(locked);
            window.set_cursor_visible(!locked);

            let mode = if locked {
                CursorGrabMode::Locked
            } else {
                CursorGrabMode::None
            };

            let _ = window.set_cursor_grab(mode);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("wowedit"))
                .unwrap(),
        );

        let wgpu = pollster::block_on(WgpuState::new(Arc::clone(&window))).unwrap();

        let egui_winit = egui_winit::State::new(
            self.egui_ctx.clone(),
            self.egui_ctx.viewport_id(),
            &window,
            None,
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &wgpu.device,
            wgpu.surface_config.format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: false,
                predictable_texture_filtering: false,
            },
        );

        self.egui_winit = Some(egui_winit);
        self.egui_renderer = Some(egui_renderer);

        // Load camera
        let terrain_center = Vec3::new(0.0, 236.0, 8533.334);
        let camera = Camera::look_at(
            terrain_center + Vec3::new(0.0, 100.0, 200.0),
            terrain_center,
        );

        let gpu_camera = GpuCamera::new(&wgpu.device);

        let storage = CompoundStorage::from_wow_install(
            "/home/arfy/Games/acwow/ChromieCraft_3.3.5a/Data",
            "enUS",
        )
        .unwrap();
        let loader = AssetLoader::new(Box::new(storage), WoWVersion::WotLK);

        // Northshire abbey - Azeroth | 32 | 48
        let adt = loader.load_adt("Azeroth", 32, 48).unwrap();

        // Load terrain
        let depth_texture = create_depth_texture(&wgpu.device, &wgpu.surface_config);
        let mipmap_generator = TerrainMipmapGenerator::new(&wgpu.device);

        let textures = loader.load_adt_textures(&adt).unwrap();
        let terrain_textures =
            TerrainTextures::new(&wgpu.device, &wgpu.queue, &textures, &mipmap_generator);

        let mut terrain_renderer = TerrainRenderer::new(
            &wgpu.device,
            wgpu.surface_config.format,
            &gpu_camera.bind_group_layout,
            &terrain_textures.bind_group_layout,
        );

        // Load m2 test
        let doodad_filenames = adt.doodad_filenames.clone().unwrap();
        let doodad_placements = adt.doodad_placements.clone().unwrap();

        // Group transforms by model path
        let mut grouped: HashMap<String, Vec<Mat4>> = HashMap::new();

        for entry in &doodad_placements {
            let path = &doodad_filenames[entry.name_id as usize];
            grouped
                .entry(path.clone())
                .or_default()
                .push(mddf_to_model_matrix(entry));
        }

        let mut m2_renderer = M2Renderer::new(
            &wgpu.device,
            wgpu.surface_config.format,
            &gpu_camera.bind_group_layout,
        );

        self.mouse_locked = Some(false);
        self.camera = Some(camera);
        self.depth_texture = Some(depth_texture);
        self.window = Some(window);
        self.terrain_textures = Some(terrain_textures);
        self.gpu_camera = Some(gpu_camera);

        let terrain = scene::terrain::Terrain::from(adt);
        terrain_renderer.load_terrain(&wgpu.device, &wgpu.queue, &terrain);

        // Load each unique model once, with all its transforms
        for (path, transforms) in &grouped {
            match loader.load_m2_resolved(path) {
                Ok(model) => {
                    m2_renderer.load(&wgpu.device, &model, transforms);
                }
                Err(e) => log::warn!("Failed to load M2 {path}: {e}"),
            }
        }

        self.wgpu = Some(wgpu);
        self.terrain_renderer = Some(terrain_renderer);
        self.m2_renderer = Some(m2_renderer);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                // This logic needs a home -> CameraController or something
                if let Some(mouse_locked) = self.mouse_locked {
                    if !mouse_locked {
                        return;
                    };

                    let sensitivity = 0.002;

                    if let Some(camera) = &mut self.camera {
                        camera.process_mouse(delta.0 as f32, delta.1 as f32, sensitivity);
                    }
                }
            }

            DeviceEvent::Removed => println!("Lost mouse focus"),
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let (Some(egui_winit), Some(window)) = (&mut self.egui_winit, &self.window) {
            let response = egui_winit.on_window_event(window, &event);
            if response.consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                println!("Device ID: {:?}", device_id);
                println!("Mouse Button: {:?}", button);
                println!("Element State: {:?}", state);
                self.set_cursor_locked(state.is_pressed());
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.handle_key(event_loop, code, key_state.is_pressed()),

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
                    Some(terrain_textures),
                    Some(m2_renderer),
                    Some(depth_texture),
                    Some(gpu_camera),
                    Some(camera),
                ) = (
                    &mut self.wgpu,
                    &self.terrain_renderer,
                    &self.terrain_textures,
                    &self.m2_renderer,
                    &self.depth_texture,
                    &self.gpu_camera,
                    &self.camera,
                ) {
                    // Make sure our camera is in position
                    let aspect =
                        wgpu.surface_config.width as f32 / wgpu.surface_config.height as f32;
                    let view_proj = camera.build_view_proj(aspect);
                    gpu_camera.update_camera(&wgpu.queue, &view_proj);

                    // egui frame
                    if let (Some(egui_winit), Some(window)) = (&mut self.egui_winit, &self.window) {
                        let raw_input = egui_winit.take_egui_input(window);
                        self.egui_ctx.begin_pass(raw_input);
                    }

                    // Draw debug UI
                    egui::Window::new("Debug").show(&self.egui_ctx, |ui| {
                        if let Some(camera) = &self.camera {
                            ui.label(format!(
                                "pos: ({:.1}, {:.1}, {:.1})",
                                camera.position.x, camera.position.y, camera.position.z,
                            ));
                            ui.label(format!(
                                "yaw: {:.2}  pitch: {:.2}",
                                camera.yaw, camera.pitch,
                            ));
                        }
                    });

                    // end egui frame
                    let egui_output = self.egui_ctx.end_pass();

                    if let (Some(egui_winit), Some(window)) = (&mut self.egui_winit, &self.window) {
                        egui_winit.handle_platform_output(window, egui_output.platform_output);
                    }

                    // Render the frame
                    match wgpu.render(
                        terrain_renderer,
                        depth_texture,
                        gpu_camera,
                        &self.egui_ctx,
                        egui_output.textures_delta,
                        egui_output.shapes,
                        egui_output.pixels_per_point,
                        &mut self.egui_renderer,
                        terrain_textures,
                        m2_renderer,
                    ) {
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
