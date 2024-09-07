use super::{
    chunk::{ChunkSection, ChunkSectionPosition, RawChunk, Volume},
    Block,
};
use noise::{Blend, Exponent, Fbm, MultiFractal, NoiseFn, Perlin};

pub const SECTION_SIZE: usize = 16;

pub trait Generate {
    fn generate_section(&self, position: ChunkSectionPosition) -> ChunkSection;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    Plains,
    Winter,
    Desert,
}

impl Biome {
    pub fn from_temperature(temperature: f64) -> Self {
        match temperature {
            0.0..=0.3 => Biome::Winter,
            0.3..=0.6 => Biome::Plains,
            0.6.. => Biome::Desert,

            _ => Biome::Plains,
        }
    }

    pub fn terrain_block(&self) -> Block {
        match self {
            Biome::Plains => Block::Grass,
            Biome::Winter => Block::Snow,
            Biome::Desert => Block::Sand,
        }
    }

    pub fn terrain_water(&self) -> Block {
        match self {
            Biome::Plains | Biome::Desert => Block::Water,
            Biome::Winter => Block::Ice,
        }
    }

    pub fn terrain_beach(&self) -> Block {
        match self {
            Biome::Plains | Biome::Desert => Block::Sand,
            Biome::Winter => Block::Gravel,
        }
    }
}

pub struct DefaultGenerator {
    noise: Box<dyn NoiseFn<f64, 2>>,
    temperature_noise: Box<dyn NoiseFn<f64, 2>>,
}

impl DefaultGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = Fbm::<Perlin>::new(seed)
            .set_frequency(0.85)
            .set_persistence(0.25)
            .set_lacunarity(2.08)
            .set_octaves(8);

        let hill_noise = Fbm::<Perlin>::new(seed)
            .set_frequency(0.45)
            .set_lacunarity(0.95)
            .set_persistence(0.65)
            .set_octaves(3);

        let temperature_noise = Fbm::<Perlin>::new(seed)
            .set_frequency(0.5)
            .set_lacunarity(0.7)
            .set_persistence(0.5)
            .set_octaves(2);

        let noise = Blend::new(noise, hill_noise.clone(), hill_noise);
        let noise = Exponent::new(noise).set_exponent(1.4);

        Self {
            noise: Box::new(noise),
            temperature_noise: Box::new(temperature_noise),
        }
    }
}

const SCALE: f64 = 64.0;
const TEMPERATURE_SCALE: f64 = 256.0;

const WATER_HEIGHT: u32 = 40;
const TERRAIN_SCALE: f64 = 48.0;
const BASE_TERRAIN_HEIGHT: u32 = 24;

impl Generate for DefaultGenerator {
    fn generate_section(&self, position: ChunkSectionPosition) -> ChunkSection {
        let mut section = ChunkSection::default();

        for x in 0..RawChunk::SIZE {
            for z in 0..RawChunk::SIZE {
                let global_x = (position.x * RawChunk::SIZE as i32) + x as i32;
                let global_z = (position.z * RawChunk::SIZE as i32) + z as i32;

                let noise_x = global_x as f64 / SCALE;
                let noise_z = global_z as f64 / SCALE;

                let temperature_x = global_x as f64 / TEMPERATURE_SCALE;
                let temperature_z = global_z as f64 / TEMPERATURE_SCALE;

                let height = self.noise.get([noise_x, noise_z]) / 2.0 + 0.5;
                let height = BASE_TERRAIN_HEIGHT + (height * TERRAIN_SCALE) as u32;

                let temperature =
                    self.temperature_noise.get([temperature_x, temperature_z]) / 2.0 + 0.5;
                let biome = Biome::from_temperature(temperature);

                for y in 0..RawChunk::SIZE * SECTION_SIZE as u32 {
                    if height > y {
                        let diff = height - y;

                        let block = match y {
                            y if diff == 1 && ((WATER_HEIGHT - 1)..=WATER_HEIGHT).contains(&y) => {
                                biome.terrain_beach()
                            }
                            _ if diff > 3 => Block::Stone,
                            _ => biome.terrain_block(),
                        };

                        section.set((x, y, z).into(), block);
                    } else if y < WATER_HEIGHT {
                        section.set((x, y, z).into(), biome.terrain_water())
                    } else {
                        continue;
                    }
                }
            }
        }

        section
    }
}
