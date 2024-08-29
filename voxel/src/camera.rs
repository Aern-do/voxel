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

    pub fn update_view_projection(mut self, view: &CameraView) -> Self {
        self.projection_matrix = view.calculate_projection_matrix();
        self.transformation_matrix = view.calculate_transformation_matrix();
        self
    }
}

#[derive(Debug)]
pub struct Camera {
    uniform: Uniform<CameraUniform>,

    controller: CameraController,
    pub view: CameraView,
}

impl Camera {
    pub const FRONT: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    pub fn new(camera_view: CameraView, graphics: &Context) -> Self {
        Self {
            view: camera_view,
            controller: CameraController::new(),

            uniform: Uniform::new(CameraUniform::new(), graphics),
        }
    }

    pub fn update(&mut self, dt: Duration, context: &Context) {
        self.controller.update_camera(&mut self.view, dt);
        self.uniform.map(
            |uniform| uniform.update_view_projection(&self.view),
            context,
        );
    }

    pub fn uniform(&self) -> &Uniform<CameraUniform> {
        &self.uniform
    }

    pub fn controller(&mut self) -> &mut CameraController {
        &mut self.controller
    }
}

impl AsBindGroup for Camera {
    type Layout = ((Vertex, Uniform<CameraUniform>),);

    fn resources(&self) -> <Self::Layout as IntoLayout>::Bindings<'_> {
        (&self.uniform,)
    }
}

#[derive(Debug, Clone)]
pub struct CameraView {
    position: Vec3,
    yaw: f32,
    pitch: f32,

    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl CameraView {
    pub fn new(
        position: Vec3,
        yaw: f32,
        pitch: f32,
        size: PhysicalSize<u32>,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            position,
            yaw,
            pitch,
            aspect: size.width as f32 / size.height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn calculate_transformation_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Mat4::look_to_rh(
            self.position,
            Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }

    pub fn calculate_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }

    pub fn front(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.front().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.front()).normalize()
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn znear(&self) -> f32 {
        self.znear
    }

    pub fn zfar(&self) -> f32 {
        self.zfar
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

    pub fn update_camera(&mut self, camera: &mut CameraView, dt: Duration) {
        const SENSITIVITY: f32 = 45.0;
        const SPEED: f32 = 48.0;

        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();

        let forward = Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();
        let horizontal = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        camera.position += forward * self.move_forward * SPEED * dt;
        camera.position += horizontal * self.move_horizontal * SPEED * dt;

        camera.yaw += (self.rotate_horizontal.to_radians()) * SENSITIVITY * dt;
        camera.pitch += (-self.rotate_vertical.to_radians()) * SENSITIVITY * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }
}
