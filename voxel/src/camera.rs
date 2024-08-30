use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use voxel_util::{AsBindGroup, Context, IntoLayout, Uniform, Vertex};
use winit::{dpi::PhysicalSize, event::ElementState, keyboard::KeyCode};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    projection_matrix: Mat4,
    transformation_matrix: Mat4,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_view_projection(
        mut self,
        projection: &Projection,
        transformation: &Transformation,
    ) -> Self {
        self.projection_matrix = projection.calculate_matrix();
        self.transformation_matrix = transformation.calculate_matrix();
        self
    }
}

#[derive(Debug)]
pub struct Camera {
    controller: CameraController,
    uniform: Uniform<CameraUniform>,

    projection: Projection,
    transformation: Transformation,
}

impl Camera {
    pub fn new(transformation: Transformation, projection: Projection, graphics: &Context) -> Self {
        Self {
            controller: CameraController::default(),
            uniform: Uniform::new(CameraUniform::new(), graphics),

            projection,
            transformation,
        }
    }

    pub fn update(&mut self, dt: Duration, context: &Context) {
        self.controller.update_camera(&mut self.transformation, dt);
        self.uniform.map(
            |uniform| uniform.update_view_projection(&self.projection, &self.transformation),
            context,
        );
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.controller.process_mouse(mouse_dx, mouse_dy)
    }

    pub fn process_keyboard(&mut self, key_code: KeyCode, state: ElementState) {
        self.controller.process_keyboard(key_code, state)
    }

    pub fn calculate_matrix(&self) -> Mat4 {
        self.projection.calculate_matrix() * self.transformation.calculate_matrix()
    }

    pub fn projection(&self) -> Projection {
        self.projection
    }

    pub fn transformation(&self) -> Transformation {
        self.transformation
    }
}

impl AsBindGroup for Camera {
    type Layout = ((Vertex, Uniform<CameraUniform>),);

    fn resources(&self) -> <Self::Layout as IntoLayout>::Bindings<'_> {
        (&self.uniform,)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transformation {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Transformation {
    pub fn new(position: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
        }
    }

    pub fn calculate_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Mat4::look_to_rh(
            self.position,
            Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(size: PhysicalSize<u32>, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: size.width as f32 / size.height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn calculate_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CameraController {
    rotate_horizontal: f32,
    rotate_vertical: f32,

    move_forward: f32,
    move_horizontal: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_keyboard(&mut self, key_code: KeyCode, state: ElementState) {
        let pressed = f32::from(state.is_pressed());

        match key_code {
            KeyCode::KeyW | KeyCode::KeyS => {
                let forward_key = f32::from(matches!(key_code, KeyCode::KeyW)) * pressed;
                let backward_key = f32::from(matches!(key_code, KeyCode::KeyS)) * pressed;
                self.move_forward = forward_key - backward_key;
            }
            KeyCode::KeyA | KeyCode::KeyD => {
                let left_key = f32::from(matches!(key_code, KeyCode::KeyA)) * pressed;
                let right_key = f32::from(matches!(key_code, KeyCode::KeyD)) * pressed;
                self.move_horizontal = right_key - left_key;
            }
            _ => {}
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn update_camera(&mut self, transformation: &mut Transformation, dt: Duration) {
        const SENSITIVITY: f32 = 45.0;
        const SPEED: f32 = 48.0;

        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = transformation.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = transformation.pitch.sin_cos();

        let forward = Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();
        let horizontal = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        transformation.position += forward * self.move_forward * SPEED * dt;
        transformation.position += horizontal * self.move_horizontal * SPEED * dt;

        transformation.yaw += (self.rotate_horizontal.to_radians()) * SENSITIVITY * dt;
        transformation.pitch += (-self.rotate_vertical.to_radians()) * SENSITIVITY * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }
}
