pub mod block;
pub mod chunk;
pub mod face;
pub mod generator;
pub mod mesher;

pub use block::{Block, Visibility};
pub use chunk::{Chunk, ChunkNeighbors};
use chunk::{ChunkSection, Volume};
pub use face::{Direction, Face};
use generator::{DefaultGenerator, Generate};
pub use mesher::{create_mesh, RawMesh};
use voxel_util::Context;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use glam::{ivec3, IVec3};

use crate::{camera::Camera, render::world_pass::ChunkBuffer};

const HORIZONTAL_RENDER_DISTANCE: i32 = 8;
const VERTICAL_RENDER_DISTANCE: i32 = 2;

#[derive(Debug, Default)]
pub struct World {
    chunks: HashMap<IVec3, Arc<Chunk>>,
    pub meshes: HashMap<IVec3, ChunkBuffer>,
}

impl World {
    const EMPTY_CHUNK: LazyLock<Arc<Chunk>> = LazyLock::new(|| Arc::new(Chunk::default()));

    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, camera: &Camera, context: &Context) {
        let origin = camera.view.position().as_ivec3() / Chunk::SIZE as i32;

        for x in -HORIZONTAL_RENDER_DISTANCE..HORIZONTAL_RENDER_DISTANCE {
            for z in -HORIZONTAL_RENDER_DISTANCE..HORIZONTAL_RENDER_DISTANCE {
                let position = ivec3(x + origin.x, 0, z + origin.z);
                if self.chunks.contains_key(&position) {
                    continue;
                }

                let mut generator = DefaultGenerator::default();
                self.generate(&mut generator, position);
            }
        }

        for x in -HORIZONTAL_RENDER_DISTANCE..HORIZONTAL_RENDER_DISTANCE {
            for z in -HORIZONTAL_RENDER_DISTANCE..HORIZONTAL_RENDER_DISTANCE {
                for y in -VERTICAL_RENDER_DISTANCE..VERTICAL_RENDER_DISTANCE {
                    let position = ivec3(x, y, z) + origin;
                    if self.meshes.contains_key(&position) {
                        continue;
                    }

                    let neighbors = ChunkNeighbors::new(position, self);

                    let mut mesh = RawMesh::default();
                    create_mesh(&neighbors, &mut mesh);

                    let mesh = ChunkBuffer::from_mesh(&mesh, position, context);
                    self.meshes.insert(origin + ivec3(x, y, z), mesh);
                }
            }
        }
    }

    pub fn get(&self, position: IVec3) -> Arc<Chunk> {
        self.chunks
            .get(&position)
            .cloned()
            .unwrap_or_else(|| World::EMPTY_CHUNK.clone())
    }

    pub fn set(&mut self, position: IVec3, chunk: Chunk) {
        self.chunks.insert(position, Arc::new(chunk));
    }

    pub fn set_many(&mut self, chunks: impl Iterator<Item = (IVec3, Chunk)>) {
        for (position, chunk) in chunks {
            self.set(position, chunk)
        }
    }

    pub fn generate<G: Generate>(&mut self, generator: &mut G, position: IVec3) {
        let mut section = ChunkSection::new(position, self);
        generator.generate(&mut section, position);

        self.set_many(section.into_iter())
    }
}
