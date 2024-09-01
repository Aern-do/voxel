use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::Instant,
};

use glam::{IVec3, Vec3};
use parking_lot::{RwLock, RwLockReadGuard};
use rayon::iter::{ParallelDrainRange, ParallelIterator};
use voxel_util::{AsBindGroup, Context};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

use crate::{
    camera::{Camera, Projection, Transformation},
    error::Error,
    render::{frustum_culling::Frustum, world_pass::ChunkBuffer, Renderer},
    world::{
        chunk::{Chunk, ChunkNeighborhood},
        meshes::create_mesh,
        World,
    },
};

enum MeshGeneratorMessage {
    InsertChunks { new_chunks: Vec<(IVec3, Chunk)> },
    SetVisible { positions: Vec<IVec3> },
}

pub struct MeshGenerator(Sender<MeshGeneratorMessage>);

impl MeshGenerator {
    fn new(sender: Sender<MeshGeneratorMessage>) -> Self {
        Self(sender)
    }

    pub fn insert_chunks(&self, new_chunks: Vec<(IVec3, Chunk)>) {
        self.0
            .send(MeshGeneratorMessage::InsertChunks { new_chunks })
            .unwrap();
    }

    pub fn set_visible(&self, positions: Vec<IVec3>) {
        self.0
            .send(MeshGeneratorMessage::SetVisible { positions })
            .unwrap();
    }
}

#[derive(Default)]
pub struct Meshes {
    generated: RwLock<HashMap<IVec3, ChunkBuffer>>,
}

impl Meshes {
    pub fn read(&self) -> RwLockReadGuard<'_, HashMap<IVec3, ChunkBuffer>> {
        self.generated.read()
    }
}

pub struct Application {
    context: Arc<Context>,
    window: Arc<Window>,

    renderer: Renderer,
    world: World,
    camera: Camera,

    meshes: Arc<Meshes>,
    mesh_generator: MeshGenerator,
    mesh_receiver: Receiver<(IVec3, ChunkBuffer)>,

    last_frame_time: Instant,
}

impl Application {
    pub async fn new(window: Window) -> Result<Self, Error> {
        let window = Arc::new(window);
        let _ = window.set_cursor_grab(CursorGrabMode::Locked);

        let context = Arc::new(Context::new(Arc::clone(&window)).await?);
        let camera = Camera::new(
            Transformation::new(Vec3::new(-2.0, 90.0, -2.0), -90.0_f32.to_radians(), 0.0),
            Projection::new(window.inner_size(), 70.0_f32.to_radians(), 0.1, 1000.0),
            &context,
        );

        let renderer = Renderer::new(camera.as_shader_resource(&context), Arc::clone(&context));
        let world = World::default();

        let (mesh_generator_sender, mesh_generator_receiver) = channel();
        let (to_generate_sender, to_generate_receiver) = channel();
        let (mesh_sender, mesh_receiver) = channel();

        let mesh_generator = MeshGenerator::new(mesh_generator_sender);
        let meshes = Arc::new(Meshes::default());
        let chunks = Arc::<RwLock<HashMap<IVec3, Chunk>>>::default();
        {
            let meshes = Arc::clone(&meshes);
            let chunks = Arc::clone(&chunks);
            thread::spawn(move || {
                for message in mesh_generator_receiver.iter() {
                    match message {
                        MeshGeneratorMessage::InsertChunks { new_chunks } => {
                            chunks.write().extend(new_chunks);
                        }

                        MeshGeneratorMessage::SetVisible { mut positions } => {
                            meshes.generated.write().retain(|mesh_position, _| {
                                positions
                                    .iter()
                                    .position(|position| position == mesh_position)
                                    .map(|index| positions.remove(index))
                                    .is_some()
                            });

                            positions.reverse();
                            to_generate_sender.send(positions).unwrap();
                        }
                    }
                }
            });
        }
        {
            let context = Arc::clone(&context);
            let chunks = Arc::clone(&chunks);

            rayon::spawn(move || {
                let mut to_generate = to_generate_receiver.recv().unwrap();
                loop {
                    to_generate = to_generate_receiver
                        .try_iter()
                        .last()
                        .unwrap_or(to_generate);

                    to_generate
                        .par_drain(to_generate.len().saturating_sub(8)..)
                        .for_each(|position| {
                            let chunks = chunks.read();
                            let neighborhood = ChunkNeighborhood::new(&chunks, position);
                            let mesh = create_mesh(neighborhood, &context);

                            mesh_sender.send((position, mesh)).unwrap();
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

            mesh_generator,
            meshes,

            last_frame_time: Instant::now(),
            mesh_receiver,
        })
    }

    pub fn draw(&mut self) {
        let frustum = Frustum::from_projection(self.camera.calculate_matrix());

        self.renderer.draw(&frustum, &self.meshes);
        self.update()
    }

    pub fn update(&mut self) {
        let delta_time = self.last_frame_time.elapsed();

        self.renderer.update(delta_time);
        self.camera.update(delta_time, &self.context);
        self.world.update(&self.camera, &self.mesh_generator);
        self.receive_meshes();

        self.last_frame_time = Instant::now();
        self.window.request_redraw();
    }

    fn receive_meshes(&self) {
        let mut meshes = self.mesh_receiver.try_iter().peekable();
        if meshes.peek().is_some() {
            self.meshes.generated.write().extend(meshes);
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.context.resize(new_size);
        self.renderer.resize(new_size);
        self.camera.resize(new_size);
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
            WindowEvent::Resized(new_size) => self.resize(new_size),
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
