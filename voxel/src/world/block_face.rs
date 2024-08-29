use glam::{vec3, IVec3, UVec3, Vec3};

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
pub struct BlockFace {
    block: Block,
    direction: Direction,
    position: UVec3,
    ao: [u32; 4],
}

impl BlockFace {
    pub fn new(block: Block, position: UVec3, ao_value: [u32; 4], direction: Direction) -> Self {
        Self {
            block,
            position,
            ao: ao_value,
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
        let make_vertex = |base_position: UVec3, index: u32| {
            let position = self.position + base_position;

            Vertex::new(
                position.to_array(),
                self.ao[index as usize],
                self.block.texture_id(),
            )
        };

        let index = [0, 1, 2, 3];

        match self.direction {
            Direction::Top => [
                make_vertex(vec3(0.0, 1.0, 0.0).as_uvec3(), index[0]),
                make_vertex(vec3(1.0, 1.0, 0.0).as_uvec3(), index[1]),
                make_vertex(vec3(1.0, 1.0, 1.0).as_uvec3(), index[2]),
                make_vertex(vec3(0.0, 1.0, 1.0).as_uvec3(), index[3]),
            ],
            Direction::Bottom => [
                make_vertex(vec3(0.0, 0.0, 0.0).as_uvec3(), index[0]),
                make_vertex(vec3(1.0, 0.0, 0.0).as_uvec3(), index[1]),
                make_vertex(vec3(1.0, 0.0, 1.0).as_uvec3(), index[2]),
                make_vertex(vec3(0.0, 0.0, 1.0).as_uvec3(), index[3]),
            ],
            Direction::Left => [
                make_vertex(vec3(0.0, 1.0, 0.0).as_uvec3(), index[0]),
                make_vertex(vec3(0.0, 1.0, 1.0).as_uvec3(), index[1]),
                make_vertex(vec3(0.0, 0.0, 1.0).as_uvec3(), index[2]),
                make_vertex(vec3(0.0, 0.0, 0.0).as_uvec3(), index[3]),
            ],
            Direction::Right => [
                make_vertex(vec3(1.0, 1.0, 0.0).as_uvec3(), index[0]),
                make_vertex(vec3(1.0, 1.0, 1.0).as_uvec3(), index[1]),
                make_vertex(vec3(1.0, 0.0, 1.0).as_uvec3(), index[2]),
                make_vertex(vec3(1.0, 0.0, 0.0).as_uvec3(), index[3]),
            ],
            Direction::Front => [
                make_vertex(vec3(0.0, 1.0, 1.0).as_uvec3(), index[0]),
                make_vertex(vec3(1.0, 1.0, 1.0).as_uvec3(), index[1]),
                make_vertex(vec3(1.0, 0.0, 1.0).as_uvec3(), index[2]),
                make_vertex(vec3(0.0, 0.0, 1.0).as_uvec3(), index[3]),
            ],
            Direction::Back => [
                make_vertex(vec3(0.0, 1.0, 0.0).as_uvec3(), index[0]),
                make_vertex(vec3(1.0, 1.0, 0.0).as_uvec3(), index[1]),
                make_vertex(vec3(1.0, 0.0, 0.0).as_uvec3(), index[2]),
                make_vertex(vec3(0.0, 0.0, 0.0).as_uvec3(), index[3]),
            ],
        }
    }
}
