use glam::{uvec3, IVec3, UVec3};

use crate::render::Vertex;

use super::block::Block;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl Direction {
    pub fn to_vec(&self) -> IVec3 {
        match self {
            Direction::Top => IVec3::Y,
            Direction::Bottom => IVec3::NEG_Y,
            Direction::Left => IVec3::NEG_X,
            Direction::Right => IVec3::X,
            Direction::Front => IVec3::Z,
            Direction::Back => IVec3::NEG_Z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Face {
    block: Block,
    direction: Direction,
    position: UVec3,
    ao: [u8; 4],
}

impl Face {
    pub fn new(block: Block, position: UVec3, ao: [u8; 4], direction: Direction) -> Self {
        Self {
            block,
            position,
            ao,
            direction,
        }
    }

    pub fn indices(index: u16) -> [u16; 6] {
        let offset = index * 4;

        [
            offset,
            1 + offset,
            2 + offset,
            2 + offset,
            3 + offset,
            offset,
        ]
    }

    pub fn vertices(&self) -> [Vertex; 4] {
        let vertices = match self.direction {
            Direction::Top => [
                uvec3(0, 1, 0),
                uvec3(1, 1, 0),
                uvec3(1, 1, 1),
                uvec3(0, 1, 1),
            ],
            Direction::Bottom => [
                uvec3(1, 0, 1),
                uvec3(1, 0, 0),
                uvec3(0, 0, 0),
                uvec3(0, 0, 1),
            ],
            Direction::Left => [
                uvec3(0, 1, 0),
                uvec3(0, 1, 1),
                uvec3(0, 0, 1),
                uvec3(0, 0, 0),
            ],
            Direction::Right => [
                uvec3(1, 1, 1),
                uvec3(1, 1, 0),
                uvec3(1, 0, 0),
                uvec3(1, 0, 1),
            ],
            Direction::Front => [
                uvec3(0, 1, 1),
                uvec3(1, 1, 1),
                uvec3(1, 0, 1),
                uvec3(0, 0, 1),
            ],
            Direction::Back => [
                uvec3(1, 1, 0),
                uvec3(0, 1, 0),
                uvec3(0, 0, 0),
                uvec3(1, 0, 0),
            ],
        };

        let mut index = 0;

        vertices.map(|vertex_position| {
            let vertex = Vertex::new(
                vertex_position + self.position,
                self.ao[index],
                self.block.texture_id(),
                self.direction as u32,
            );
            index += 1;

            vertex
        })
    }
}
