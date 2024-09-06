use std::{
    collections::HashMap,
    ops::{Add, Index, IndexMut},
};

use glam::{uvec3, IVec3, UVec3};

use super::Block;

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

pub type ChunkSlice = [[Block; CHUNK_SIZE]; CHUNK_SIZE];

#[derive(Default, Clone)]
pub struct RawChunk {
    pub stack: [ChunkSlice; CHUNK_SIZE],
}

impl RawChunk {
    pub fn iter(&self) -> impl Iterator<Item = Block> + '_ {
        self.stack.iter().copied().flatten().flatten()
    }

    pub fn iter_enumerate(&self) -> impl Iterator<Item = (UVec3, Block)> + '_ {
        self.stack.iter().enumerate().flat_map(|(y, blocks_xz)| {
            let y = y as u32;
            blocks_xz.iter().enumerate().flat_map(move |(x, blocks_z)| {
                let x = x as u32;
                blocks_z.iter().copied().enumerate().map(move |(z, block)| {
                    let z = z as u32;
                    (uvec3(x, y, z), block)
                })
            })
        })
    }
}

impl Index<UVec3> for RawChunk {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        &self.stack[position.y as usize][position.x as usize][position.z as usize]
    }
}

impl IndexMut<UVec3> for RawChunk {
    fn index_mut(&mut self, position: UVec3) -> &mut Self::Output {
        &mut self.stack[position.y as usize][position.x as usize][position.z as usize]
    }
}

impl Volume for RawChunk {
    const SIZE: u32 = CHUNK_SIZE as u32;
}

pub type Chunk = Box<RawChunk>;

#[derive(Default, Clone, Copy)]
pub struct ChunkOrAir<'s>(pub Option<&'s Chunk>);

impl<'s> ChunkOrAir<'s> {
    pub fn new(chunk: &'s Chunk) -> Self {
        Self(Some(chunk))
    }
}

impl Index<UVec3> for ChunkOrAir<'_> {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        if let Some(chunk) = self.0 {
            &chunk[position]
        } else {
            &Block::Air
        }
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
pub struct ChunkNeighborhood<'s> {
    chunks: &'s HashMap<IVec3, Chunk>,
    center: IVec3,
}

impl<'s> ChunkNeighborhood<'s> {
    pub fn new(chunks: &'s HashMap<IVec3, Chunk>, center: IVec3) -> Self {
        Self { chunks, center }
    }

    pub fn get(&self, position: UVec3) -> Block {
        const MAX: u32 = RawChunk::SIZE + 1;

        let center = self.chunks.get(&self.center).unwrap();
        let neighbors = OFFSETS.map(|offset| self.center + offset).map(|position| {
            self.chunks
                .get(&position)
                .map(ChunkOrAir::new)
                .unwrap_or_default()
        });

        match (position.x, position.y, position.z) {
            (1..=RawChunk::SIZE, 1..=RawChunk::SIZE, 1..=RawChunk::SIZE) => {
                center[(position.x - 1, position.y - 1, position.z - 1).into()]
            }
            (MAX, 1..=RawChunk::SIZE, 1..=RawChunk::SIZE) => {
                neighbors[0][(0, position.y - 1, position.z - 1).into()]
            }
            (0, 1..=RawChunk::SIZE, 1..=RawChunk::SIZE) => {
                neighbors[1][(RawChunk::SIZE - 1, position.y - 1, position.z - 1).into()]
            }
            (1..=RawChunk::SIZE, MAX, 1..=RawChunk::SIZE) => {
                neighbors[2][(position.x - 1, 0, position.z - 1).into()]
            }
            (1..=RawChunk::SIZE, 0, 1..=RawChunk::SIZE) => {
                neighbors[3][(position.x - 1, RawChunk::SIZE - 1, position.z - 1).into()]
            }
            (1..=RawChunk::SIZE, 1..=RawChunk::SIZE, MAX) => {
                neighbors[4][(position.x - 1, position.y - 1, 0).into()]
            }
            (1..=RawChunk::SIZE, 1..=RawChunk::SIZE, 0) => {
                neighbors[5][(position.x - 1, position.y - 1, RawChunk::SIZE - 1).into()]
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

impl From<(i32, i32)> for ChunkSectionPosition {
    fn from((x, z): (i32, i32)) -> Self {
        Self::new(x, z)
    }
}

impl From<IVec3> for ChunkSectionPosition {
    fn from(IVec3 { x, z, .. }: IVec3) -> Self {
        Self::new(x, z)
    }
}

impl Add for ChunkSectionPosition {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            z: self.z + other.z,
        }
    }
}

#[derive(Default, Clone)]
pub struct ChunkSection {
    chunks: [Option<Chunk>; SECTION_SIZE],
}

impl ChunkSection {
    pub fn into_chunks(self) -> impl Iterator<Item = (usize, Chunk)> {
        self.chunks
            .into_iter()
            .enumerate()
            .filter_map(|(position, chunk)| {
                let chunk = chunk?;
                if chunk.iter().any(|block| block != Block::Air) {
                    Some((position, chunk))
                } else {
                    None
                }
            })
    }

    pub fn set(&mut self, position: UVec3, block: Block) {
        assert!(block != Block::Air);

        let index = (position.y / RawChunk::SIZE) as usize;
        let position = position.with_y(position.y % RawChunk::SIZE);

        let chunk = self.chunks[index].get_or_insert_with(Default::default);
        chunk[position] = block;
    }
}

impl Index<UVec3> for ChunkSection {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        let index = (position.y / RawChunk::SIZE) as usize;
        let position = position.with_y(position.y % RawChunk::SIZE);

        let Some(chunk) = &self.chunks[index] else {
            return &Block::Air;
        };
        &chunk[position]
    }
}
