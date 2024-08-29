use glam::UVec3;
use std::mem::size_of;
use voxel_util::VertexLayout;
use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex(u32);

impl Vertex {
    const ATTRIBUTES: [VertexAttribute; 1] = vertex_attr_array![0 => Uint32];

    pub const fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Vertex::ATTRIBUTES,
        }
    }

    pub fn new(position: UVec3, ao: u8, texture_id: u32, direction: u32) -> Self {
        let value = (position.x << 27)
            | (position.y << 22)
            | (position.z << 17)
            | ((ao as u32) << 15)
            | (texture_id << 9)
            | (direction << 6);

        Self(value)
    }
}

impl VertexLayout for Vertex {
    fn vertex_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Vertex::ATTRIBUTES,
        }
    }
}
