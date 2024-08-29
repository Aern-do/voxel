use std::ops::{Index, IndexMut};

use glam::{uvec3, IVec3, UVec3};

use crate::world::EMPTY_CHUNK;

use super::{Block, World};

pub trait Volume {
    const SIZE: u32;

    fn linearize(position: impl Into<(u32, u32, u32)>) -> u32 {
        let (x, y, z) = position.into();
        x + (y * Self::SIZE) + (z * Self::SIZE * Self::SIZE)
    }

    fn delinearize(mut index: u32) -> UVec3 {
        let z = index / (Self::SIZE * Self::SIZE);
        index -= z * (Self::SIZE * Self::SIZE);

        let y = index / Self::SIZE;
        index -= y * Self::SIZE;

        let x = index;

        uvec3(x, y, z)
    }
}

pub const CHUNK_SIZE: usize = 16;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub blocks: Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
}

impl Volume for Chunk {
    const SIZE: u32 = CHUNK_SIZE as u32;
}

impl Index<UVec3> for Chunk {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        &self.blocks[position.x as usize][position.z as usize][position.y as usize]
    }
}

impl IndexMut<UVec3> for Chunk {
    fn index_mut(&mut self, position: UVec3) -> &mut Self::Output {
        &mut self.blocks[position.x as usize][position.z as usize][position.y as usize]
    }
}

const OFFSETS: [IVec3; 6] = [
    IVec3::X,
    IVec3::NEG_X,
    IVec3::Y,
    IVec3::NEG_Y,
    IVec3::Z,
    IVec3::NEG_Z,
];

#[derive(Clone, Copy)]
pub struct ChunkNeighborhood<'w> {
    world: &'w World,
    center: IVec3,
}

impl<'w> ChunkNeighborhood<'w> {
    pub fn new(world: &'w World, center: IVec3) -> Self {
        Self { world, center }
    }
}

impl Index<UVec3> for ChunkNeighborhood<'_> {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        const MAX: u32 = Chunk::SIZE + 1;

        let center = &self.world.chunks[&self.center];
        let neighbors = OFFSETS
            .map(|offset| self.center + offset)
            .map(|position| self.world.chunks.get(&position).unwrap_or(&EMPTY_CHUNK));

        match (position.x, position.y, position.z) {
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                &center[(position.x - 1, position.y - 1, position.z - 1).into()]
            }
            (MAX, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                &neighbors[0][(0, position.y - 1, position.z - 1).into()]
            }
            (0, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                &neighbors[1][(Chunk::SIZE - 1, position.y - 1, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, MAX, 1..=Chunk::SIZE) => {
                &neighbors[2][(position.x - 1, 0, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, 0, 1..=Chunk::SIZE) => {
                &neighbors[3][(position.x - 1, Chunk::SIZE - 1, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, MAX) => {
                &neighbors[4][(position.x - 1, position.y - 1, 0).into()]
            }
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, 0) => {
                &neighbors[5][(position.x - 1, position.y - 1, Chunk::SIZE - 1).into()]
            }
            (_, _, _) => &Block::Air,
        }
    }
}

pub const SECTION_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkSectionPosition {
    pub x: i32,
    pub z: i32,
}

impl ChunkSectionPosition {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub const fn with_y(self, y: i32) -> IVec3 {
        IVec3 {
            x: self.x,
            y,
            z: self.z,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChunkSection {
    pub chunks: [Chunk; SECTION_SIZE],
}

impl Index<UVec3> for ChunkSection {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        let index = position.y / Chunk::SIZE;
        let chunk = &self.chunks[index as usize];

        &chunk[position - (index * Chunk::SIZE)]
    }
}

impl IndexMut<UVec3> for ChunkSection {
    fn index_mut(&mut self, position: UVec3) -> &mut Self::Output {
        let index = position.y / Chunk::SIZE;
        let chunk = &mut self.chunks[index as usize];

        &mut chunk[position - (UVec3::Y * (index * Chunk::SIZE))]
    }
}
