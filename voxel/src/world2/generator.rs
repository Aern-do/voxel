use glam::IVec3;
use noise::{NoiseFn, Perlin};

use super::{chunk::{ChunkSection, Volume}, Block, Chunk};

pub trait Generate {
    fn generate(&mut self, chunk: &mut ChunkSection, position: IVec3);
}

#[derive(Debug, Default, Clone)]
pub struct DefaultGenerator {
    perlin: Perlin,
}

impl DefaultGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Generate for DefaultGenerator {
    fn generate(&mut self, chunk: &mut ChunkSection, position: IVec3) {
        for x in 0..Chunk::SIZE {
            for z in 0..Chunk::SIZE {
                const SCALE: f64 = 64.0;
        
                let global_x = (position.x * Chunk::SIZE as i32) + x as i32;
                let global_z = (position.z * Chunk::SIZE as i32) + z as i32;
        
                let noise_x = global_x as f64 / SCALE;
                let noise_z = global_z as f64 / SCALE;

                let height = self.perlin.get([noise_x, noise_z]) / 2.0 + 0.5;
                let height = 48 + (height * 48.0) as u32;

                for y in 0..height {
                    let dy = height - y;

                    if dy == 1 {
                        if y <= 64 && y >= 63 {
                            chunk[(x, y, z).into()] = Block::Sand;
                        } else if y < 63 {
                            chunk[(x, y, z).into()] = Block::Stone;
                        } else {
                            chunk[(x, y, z).into()] = Block::Grass;
                        }
                    } else if dy < 5 {
                        if y < 64 {
                            chunk[(x, y, z).into()] = Block::Stone;
                        } else {
                            chunk[(x, y, z).into()] = Block::Dirt;
                        }
                    } else {
                        chunk[(x, y, z).into()] = Block::Stone;
                    }
                }

                for y in height..64 {
                    chunk[(x, y, z).into()] = Block::Water;
                }
            }
        }
    }
}
