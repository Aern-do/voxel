use std::{
    collections::HashMap,
    ops::Index,
    sync::{Arc, OnceLock},
};

use glam::{uvec3, IVec2};
use itertools::iproduct;
use noise::{NoiseFn, Perlin};

use super::{
    block::Block,
    chunk::{Chunk, ChunkPosition, LocalPosition, Volume},
};

#[derive(Debug, Default, Clone)]
pub struct ChunkMap {
    chunks: HashMap<ChunkPosition, Arc<Chunk>>,
}

impl ChunkMap {
    const EMPTY_CHUNK: OnceLock<Arc<Chunk>> = OnceLock::new();

    pub fn new() -> Self {
        Self::default()
    }

    pub fn chunk_at(&self, position: &ChunkPosition) -> Arc<Chunk> {
        self.chunks
            .get(position)
            .cloned()
            .unwrap_or_else(|| ChunkMap::empty_chunk())
    }

    pub fn update_chunk_at(&mut self, position: ChunkPosition, chunk: Chunk) {
        self.chunks.insert(position, Arc::new(chunk));
    }

    pub fn generate(&mut self, position: ChunkPosition, noise: &mut Perlin) {
        let position = position.0;
        let mut chunk = Chunk::new();

        for x in 0..Chunk::SIZE {
            for z in 0..Chunk::SIZE {
                const SCALE: f64 = 16.0;
                const WATER_HEIGHT: u32 = 5;
        
                let global_x = (position.x * Chunk::SIZE as i32) + x as i32;
                let global_z = (position.y * Chunk::SIZE as i32) + z as i32;
        
                let noise_x = global_x as f64 / SCALE;
                let noise_z = global_z as f64 / SCALE;
        
                let height = (noise.get([noise_x, noise_z]) * 8.0 + 8.0).round() as u32;
                let height = height.clamp(0, 16);
        
                for y in 0..Chunk::SIZE {
                    if y >= height {
                        continue;
                    }
        
                    if y <= WATER_HEIGHT {
                        // Additional noise to determine if the block should be water
                        let puddle_chance = noise.get([noise_x, noise_z, y as f64 * 0.1]);
                        if puddle_chance > 0.7 {
                            chunk[uvec3(x, y, z).into()] = Block::Water;
                        } else {
                            chunk[uvec3(x, y, z).into()] = Block::Grass;
                        }
                    } else {
                        chunk[uvec3(x, y, z).into()] = Block::Grass;
                    }
                }
            }
        }
        self.update_chunk_at(ChunkPosition(position), chunk);
    }

    fn empty_chunk() -> Arc<Chunk> {
        ChunkMap::EMPTY_CHUNK
            .get_or_init(|| Arc::new(Chunk::new()))
            .clone()
    }
}

#[derive(Debug, Clone)]
pub struct ChunkBoundary {
    center: Arc<Chunk>,
    neighbors: [Arc<Chunk>; 4],
}

impl ChunkBoundary {
    const MAX: u32 = Chunk::SIZE + 1;

    pub fn new(chunk_map: &ChunkMap, center: ChunkPosition) -> Self {
        let center = center.0;

        let negative_y = chunk_map.chunk_at(&ChunkPosition(center + IVec2::NEG_Y));
        let negative_x = chunk_map.chunk_at(&ChunkPosition(center + IVec2::NEG_X));
        let positive_y = chunk_map.chunk_at(&ChunkPosition(center + IVec2::Y));
        let positive_x = chunk_map.chunk_at(&ChunkPosition(center + IVec2::X));
        let center = chunk_map.chunk_at(&ChunkPosition(center));

        Self {
            center: center,
            neighbors: [negative_y, positive_y, negative_x, positive_x],
        }
    }

    pub fn meshing_range() -> impl Iterator<Item = (u32, u32, u32)> {
        iproduct!(1..Chunk::SIZE + 1, 0..Chunk::SIZE, 1..Chunk::SIZE + 1)
    }
}

impl<P> Index<P> for ChunkBoundary where LocalPosition: From<P> {
    type Output = Block;

    fn index(&self, position: P) -> &Self::Output {
        let position = LocalPosition::from(position);
        let position = position.0;
        
        match (position.x, position.z) {
            (1..=Chunk::SIZE, 1..=Chunk::SIZE) => {
                &self.center[uvec3(position.x - 1, position.y, position.z - 1).into()]
            }
            (1..=Chunk::SIZE, 0) => {
                &self.neighbors[0][uvec3(position.x - 1, position.y, Chunk::SIZE - 1).into()]
            }
            (1..=Chunk::SIZE, ChunkBoundary::MAX) => {
                &self.neighbors[1][uvec3(position.x - 1, position.y, 0).into()]
            }
            (0, 1..=Chunk::SIZE) => {
                &self.neighbors[2][uvec3(Chunk::SIZE - 1, position.y, position.z - 1).into()]
            }
            (ChunkBoundary::MAX, 1..=Chunk::SIZE) => {
                &self.neighbors[3][uvec3(0, position.y, position.z - 1).into()]
            }
            (_, _) => &Block::Air,
        }
    }
}
