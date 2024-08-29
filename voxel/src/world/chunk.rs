use std::ops::{Index, IndexMut};

use derive_more::{From, Into};
use glam::{uvec3, IVec2, IVec3, UVec3};

use super::block::Block;

pub trait Volume {
    const SIZE: u32;
    const VOLUME: usize = (Self::SIZE as usize).pow(3);
    type Position;

    fn linearize(position: Self::Position) -> u32
    where
        UVec3: From<Self::Position>,
    {
        let position = UVec3::from(position);

        position.x + (position.y * Self::SIZE) + (position.z * Self::SIZE * Self::SIZE)
    }

    fn delinearize(mut index: u32) -> Self::Position
    where
        Self::Position: From<UVec3>,
    {
        let z = index / (Self::SIZE * Self::SIZE);
        index -= z * (Self::SIZE * Self::SIZE);

        let y = index / Self::SIZE;
        index -= y * Self::SIZE;

        let x = index;

        Self::Position::from(uvec3(x, y, z))
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    blocks: Box<[Block; Chunk::VOLUME]>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: Box::new([Block::default(); Chunk::VOLUME]),
        }
    }
}

impl Volume for Chunk {
    const SIZE: u32 = 16;
    type Position = LocalPosition;
}

impl Index<LocalPosition> for Chunk {
    type Output = Block;

    fn index(&self, position: LocalPosition) -> &Self::Output {
        let index = Self::linearize(position);

        &self.blocks[index as usize]
    }
}

impl From<(u32, u32, u32)> for LocalPosition {
    fn from(value: (u32, u32, u32)) -> Self {
        Self(uvec3(value.0, value.1, value.2))
    }
}

impl IndexMut<LocalPosition> for Chunk {
    fn index_mut(&mut self, position: LocalPosition) -> &mut Self::Output {
        let index = Self::linearize(position);

        &mut self.blocks[index as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Into, From, Hash)]
pub struct LocalPosition(pub UVec3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalPosition(pub IVec3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition(pub IVec2);

impl From<GlobalPosition> for LocalPosition {
    fn from(position: GlobalPosition) -> Self {
        let position = position.0;
        let mask = (Chunk::SIZE - 1) as i32;

        LocalPosition(UVec3::new(
            (position.x & mask) as u32,
            position.y as u32,
            (position.z & mask) as u32,
        ))
    }
}
