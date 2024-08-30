use std::ops::{Index, IndexMut};

use glam::{uvec3, IVec3, UVec3};

use crate::world::EMPTY_CHUNK;

use super::{Block, ChunksInner};

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
    blocks: Option<Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>>,
}

impl Volume for Chunk {
    const SIZE: u32 = CHUNK_SIZE as u32;
}

impl Chunk {
    fn is_empty(&self) -> bool {
        !self.blocks.as_ref().is_some_and(|blocks| {
            blocks
                .iter()
                .copied()
                .flatten()
                .flatten()
                .any(|block| block != Block::Air)
        })
    }
}

impl Index<UVec3> for Chunk {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        if let Some(blocks) = &self.blocks {
            &blocks[position.x as usize][position.z as usize][position.y as usize]
        } else {
            &Block::Air
        }
    }
}

impl IndexMut<UVec3> for Chunk {
    fn index_mut(&mut self, position: UVec3) -> &mut Self::Output {
        &mut self.blocks.get_or_insert_with(Default::default)[position.x as usize]
            [position.z as usize][position.y as usize]
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

pub struct ChunkNeighborhood<'s> {
    chunks: &'s ChunksInner,
    center: IVec3,
}

impl<'s> ChunkNeighborhood<'s> {
    pub fn new(chunks: &'s ChunksInner, center: IVec3) -> Self {
        Self { chunks, center }
    }

    pub fn get(&self, position: UVec3) -> Block {
        const MAX: u32 = Chunk::SIZE + 1;

        let center = self.chunks.get(&self.center).unwrap();
        let neighbors = OFFSETS
            .map(|offset| self.center + offset)
            .map(|position| self.chunks.get(&position).unwrap_or(&EMPTY_CHUNK));

        match (position.x, position.y, position.z) {
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                center[(position.x - 1, position.y - 1, position.z - 1).into()]
            }
            (MAX, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                neighbors[0][(0, position.y - 1, position.z - 1).into()]
            }
            (0, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                neighbors[1][(Chunk::SIZE - 1, position.y - 1, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, MAX, 1..=Chunk::SIZE) => {
                neighbors[2][(position.x - 1, 0, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, 0, 1..=Chunk::SIZE) => {
                neighbors[3][(position.x - 1, Chunk::SIZE - 1, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, MAX) => {
                neighbors[4][(position.x - 1, position.y - 1, 0).into()]
            }
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, 0) => {
                neighbors[5][(position.x - 1, position.y - 1, Chunk::SIZE - 1).into()]
            }
            (_, _, _) => Block::Air,
        }
    }

    pub fn center(&self) -> IVec3 {
        self.center
    }
}

pub const SECTION_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    pub const fn adjacent(self) -> [ChunkSectionPosition; 4] {
        [
            Self::new(self.x - 1, self.z),
            Self::new(self.x + 1, self.z),
            Self::new(self.x, self.z - 1),
            Self::new(self.x, self.z + 1),
        ]
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChunkSection {
    chunks: [Chunk; SECTION_SIZE],
}

impl ChunkSection {
    pub fn into_chunks(self) -> impl Iterator<Item = (usize, Chunk)> {
        self.chunks
            .into_iter()
            .enumerate()
            .filter(|(_, chunk)| !chunk.is_empty())
    }
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
