use noise::{NoiseFn, Perlin};

use super::{
    chunk::{ChunkSection, ChunkSectionPosition, Volume},
    Block, Chunk,
};

pub const SECTION_SIZE: usize = 16;

pub trait Generate {
    fn generate_section(&self, position: ChunkSectionPosition) -> ChunkSection;
}

#[derive(Debug, Default, Clone)]
pub struct DefaultGenerator {
    perlin: Perlin,
}

impl Generate for DefaultGenerator {
    fn generate_section(&self, position: ChunkSectionPosition) -> ChunkSection {
        let mut section = ChunkSection::default();

        for x in 0..Chunk::SIZE {
            for z in 0..Chunk::SIZE {
                const SCALE: f64 = 64.0;

                let global_x = (position.x * Chunk::SIZE as i32) + x as i32;
                let global_z = (position.z * Chunk::SIZE as i32) + z as i32;

                let noise_x = global_x as f64 / SCALE;
                let noise_z = global_z as f64 / SCALE;

                let height = self.perlin.get([noise_x, noise_z]) / 2.0 + 0.5;
                let height = 48 + (height * 48.0) as u32;

                for y in 0..Chunk::SIZE * SECTION_SIZE as u32 {
                    section[(x, y, z).into()] = {
                        if height > y {
                            let dy = height - y;
                            if dy == 1 {
                                if (63..=64).contains(&y) {
                                    Block::Sand
                                } else if y < 63 {
                                    Block::Stone
                                } else {
                                    Block::Grass
                                }
                            } else if dy < 5 {
                                if y < 64 {
                                    Block::Stone
                                } else {
                                    Block::Dirt
                                }
                            } else {
                                Block::Stone
                            }
                        } else if y < 64 {
                            Block::Water
                        } else {
                            continue;
                        }
                    }
                }
            }
        }

        section
    }
}
