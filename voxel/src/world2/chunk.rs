use std::{
    array, iter,
    ops::{Index, IndexMut},
    sync::Arc,
};

use glam::{uvec3, IVec3, UVec3};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    is_empty: bool,
    blocks: Box<[Block; 4096]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            is_empty: true,
            blocks: Box::new([Block::default(); 4096]),
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty(&self) -> bool {
        self.is_empty
    }
}

impl Volume for Chunk {
    const SIZE: u32 = 16;
}

impl Index<UVec3> for Chunk {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        &self.blocks[Chunk::linearize(position) as usize]
    }
}

impl IndexMut<UVec3> for Chunk {
    fn index_mut(&mut self, position: UVec3) -> &mut Self::Output {
        self.is_empty = false;
        &mut self.blocks[Chunk::linearize(position) as usize]
    }
}

pub struct ChunkNeighbors {
    center: Arc<Chunk>,
    neighbors: [Arc<Chunk>; 6],
}

impl ChunkNeighbors {
    pub fn new(center: IVec3, world: &World) -> Self {
        let x_pos = world.get(center + IVec3::X);
        let x_neg = world.get(center + IVec3::NEG_X);

        let y_pos = world.get(center + IVec3::Y);
        let y_neg = world.get(center + IVec3::NEG_Y);

        let z_pos = world.get(center + IVec3::Z);
        let z_neg = world.get(center + IVec3::NEG_Z);

        Self {
            center: world.get(center),
            neighbors: [x_pos, x_neg, y_pos, y_neg, z_pos, z_neg],
        }
    }

    pub fn meshing_range() -> impl Iterator<Item = (u32, u32, u32)> {
        let x = 1..Chunk::SIZE + 1;
        let y = 1..Chunk::SIZE + 1;
        let z = 1..Chunk::SIZE + 1;

        x.flat_map(move |i| iter::repeat(i).zip(y.clone()))
            .flat_map(move |i| iter::repeat(i).zip(z.clone()))
            .map(|((x, y), z)| (x, y, z))
    }
}

impl Index<UVec3> for ChunkNeighbors {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        const MAX: u32 = Chunk::SIZE + 1;

        match (position.x, position.y, position.z) {
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                &self.center[(position.x - 1, position.y - 1, position.z - 1).into()]
            }
            (MAX, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                &self.neighbors[0][(0, position.y - 1, position.z - 1).into()]
            }
            (0, 1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                &self.neighbors[1][(Chunk::SIZE - 1, position.y - 1, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, MAX, 1..=Chunk::SIZE) => {
                &self.neighbors[2][(position.x - 1, 0, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, 0, 1..=Chunk::SIZE) => {
                &self.neighbors[3][(position.x - 1, Chunk::SIZE - 1, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, MAX) => {
                &self.neighbors[4][(position.x - 1, position.y - 1, 0).into()]
            }
            (1..=Chunk::SIZE, 1..=Chunk::SIZE, 0) => {
                &self.neighbors[5][(position.x - 1, position.y - 1, Chunk::SIZE - 1).into()]
            }
            (_, _, _) => &Block::Air,
        }
    }
}

const SECTION_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct ChunkSection {
    sections: [Chunk; SECTION_SIZE],
    base_position: IVec3,
}

impl ChunkSection {
    pub fn new(base_position: IVec3, world: &World) -> Self {
        let sections = array::from_fn(|index| {
            let section_position = base_position + (IVec3::Y * index as i32);
            Arc::unwrap_or_clone(world.get(section_position))
        });

        Self {
            sections,
            base_position,
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = (IVec3, Chunk)> {
        let mut index = 0;
        self.sections
            .map(|chunk| {
                let position = self.base_position + (IVec3::Y * index);
                index += 1;

                (position, chunk)
            })
            .into_iter()
    }
}

impl Index<UVec3> for ChunkSection {
    type Output = Block;

    fn index(&self, position: UVec3) -> &Self::Output {
        let section_index = position.y / Chunk::SIZE;
        let section = &self.sections[section_index as usize];

        &section[position - (section_index * Chunk::SIZE)]
    }
}

impl IndexMut<UVec3> for ChunkSection {
    fn index_mut(&mut self, position: UVec3) -> &mut Self::Output {
        let section_index = position.y / Chunk::SIZE;
        let section = &mut self.sections[section_index as usize];

        &mut section[position - (UVec3::Y * (section_index * Chunk::SIZE))]
    }
}
