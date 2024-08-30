use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
    time::Instant,
};

use glam::{IVec3, Vec3};
use voxel_util::{AsBindGroup, Context};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

use crate::{
    camera::{Camera, Projection, Transformation},
    error::Error,
    render::{frustum_culling::Frustum, Renderer},
    world::{
        chunk::ChunkNeighborhood,
        meshes::{create_mesh, Meshes, MeshesMessage},
        World,
    },
};

pub struct MeshUpdater {
    position_sender: Sender<IVec3>,
    meshes_sender: Sender<MeshesMessage>,
}

impl MeshUpdater {
    pub fn generate(&self, position: IVec3) {
        self.position_sender.send(position).unwrap();
    }

    pub fn ungenerate(&self, position: IVec3) {
        self.meshes_sender
            .send(MeshesMessage::Ungenerate { position })
            .unwrap();
    }
}

pub struct Application {
    context: Arc<Context>,
    window: Arc<Window>,

    renderer: Renderer,
    world: World,
    camera: Camera,

    mesh_updater: MeshUpdater,
    meshes: Meshes,

    last_frame_time: Instant,
}

impl Application {
    pub async fn new(window: Window) -> Result<Self, Error> {
        let window = Arc::new(window);
        let _ = window.set_cursor_grab(CursorGrabMode::Locked);

        let context = Arc::new(Context::new(Arc::clone(&window)).await?);
        let camera = Camera::new(
            Transformation::new(Vec3::new(-2.0, 90.0, -2.0), -90.0_f32.to_radians(), 0.0),
            Projection::new(window.inner_size(), 45.0_f32.to_radians(), 0.1, 1000.0),
            &context,
        );

        let renderer = Renderer::new(camera.as_shader_resource(&context), Arc::clone(&context));
        let world = World::default();

        let (position_sender, position_receiver) = channel();
        let (meshes_sender, mesh_receiver) = channel();
        let meshes = Meshes::new(mesh_receiver);
        {
            let chunks = world.chunks.clone();
            let context = Arc::clone(&context);
            let meshes_sender = meshes_sender.clone();

            thread::spawn(move || {
                while let Ok(position) = position_receiver.recv() {
                    let chunks = chunks.clone();
                    let context = Arc::clone(&context);
                    let meshes_sender = meshes_sender.clone();

                    rayon::spawn_fifo(move || {
                        let mesh = {
                            let chunks = chunks.read();
                            let neighborhood = ChunkNeighborhood::new(&chunks, position);
                            create_mesh(&neighborhood, &context)
                        };
                        meshes_sender
                            .send(MeshesMessage::Insert { position, mesh })
                            .unwrap();
                    });
                }
            });
        }

        Ok(Self {
            context,
            window,

            renderer,
            world,
            camera,

            mesh_updater: MeshUpdater {
                position_sender,
                meshes_sender,
            },
            meshes,

            last_frame_time: Instant::now(),
        })
    }

    pub fn draw(&mut self) {
        let frustum = Frustum::from_projection(self.camera.calculate_matrix());

        self.meshes.receive();
        self.renderer.draw(&frustum, &self.meshes);
        self.update()
    }

    pub fn update(&mut self) {
        let delta_time = self.last_frame_time.elapsed();

        self.renderer.update(delta_time);
        self.camera.update(delta_time, &self.context);
        self.world.update(&self.camera, &self.mesh_updater);

        self.last_frame_time = Instant::now();
        self.window.request_redraw();
    }

    pub fn keyboard_input(&mut self, key_code: KeyCode, state: ElementState) {
        self.camera.process_key(key_code, state);
    }

    pub fn mouse_motion(&mut self, dx: f64, dy: f64) {
        self.camera.process_mouse(dx, dy);
    }

    pub fn mouse_moved(&self) {
        let size = self.window.inner_size();
        let _ = self
            .window
            .set_cursor_position(PhysicalPosition::new(size.width / 2, size.height / 2));
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, _: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => self.keyboard_input(key_code, state),
            WindowEvent::CursorMoved { .. } => self.mouse_moved(),
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.mouse_motion(delta.0, delta.1)
        }
    }
}
