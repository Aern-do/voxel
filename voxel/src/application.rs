use std::{sync::Arc, time::Instant};

use glam::Vec3;
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
    world::World,
};

pub struct Application {
    context: Arc<Context>,
    window: Arc<Window>,

    renderer: Renderer,
    world: World,
    camera: Camera,

    last_frame_time: Instant,
}

impl Application {
    pub async fn new(window: Window) -> Result<Self, Error> {
        let window = Arc::new(window);
        let _ = window.set_cursor_grab(CursorGrabMode::Locked);

        let context = Arc::new(Context::new(window.clone()).await?);
        let camera = Camera::new(
            Transformation::new(
                Vec3::new(-2.0, 90.0, -2.0),
                -90.0_f32.to_radians(),
                45.0_f32.to_radians(),
            ),
            Projection::new(window.inner_size(), 45.0_f32.to_radians(), 0.1, 1000.0),
            &context,
        );

        let renderer = Renderer::new(camera.as_shader_resource(&context), context.clone());
        let world = World::new();

        Ok(Self {
            renderer,
            camera,
            world,

            last_frame_time: Instant::now(),
            window,
            context,
        })
    }

    pub fn draw(&mut self) {
        let frustum = Frustum::from_projection(self.camera.calculate_matrix());

        self.renderer.draw(&frustum, &self.world);
        self.update()
    }

    pub fn update(&mut self) {
        let delta_time = self.last_frame_time.elapsed();

        self.renderer.update(delta_time);
        self.camera.update(delta_time, &self.context);
        self.world.update(&self.camera, &self.context);

        self.last_frame_time = Instant::now();
        self.window.request_redraw();
    }

    pub fn keyboard_input(&mut self, key_code: KeyCode, state: ElementState) {
        self.camera.process_keyboard(key_code, state);
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
