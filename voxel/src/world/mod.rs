pub mod block;
pub mod chunk;
pub mod face;
pub mod generator;
pub mod mesher;

pub use block::{Block, Visibility};
pub use chunk::Chunk;
use chunk::SECTION_SIZE;
use chunk::{ChunkNeighborhood, ChunkSectionPosition, Volume};
pub use face::{Direction, Face};
use generator::{DefaultGenerator, Generate};
pub use mesher::{create_mesh, RawMesh};
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use std::iter;
use voxel_util::Context;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

use glam::IVec3;

use crate::{camera::Camera, render::world_pass::ChunkBuffer};

const HORIZONTAL_RENDER_DISTANCE: i32 = 8;
const VERTICAL_RENDER_DISTANCE: i32 = 4;

pub static EMPTY_CHUNK: LazyLock<Chunk> = LazyLock::new(|| Chunk {
    blocks: Default::default(),
});

#[derive(Debug, Default)]
pub struct World {
    chunks: HashMap<IVec3, Chunk>,
    generated_sections: HashSet<ChunkSectionPosition>,
    generator: DefaultGenerator,
    pub meshes: HashMap<IVec3, ChunkBuffer>,
}

static GENERATING_SECTIONS: LazyLock<Box<[ChunkSectionPosition]>> = LazyLock::new(|| {
    (-HORIZONTAL_RENDER_DISTANCE..HORIZONTAL_RENDER_DISTANCE + 1)
        .flat_map(|x| {
            iter::repeat(x).zip(-HORIZONTAL_RENDER_DISTANCE..HORIZONTAL_RENDER_DISTANCE + 1)
        })
        .map(|(x, z)| ChunkSectionPosition::new(x, z))
        .collect()
});

static RENDERING_CHUNKS_OFFSETS: LazyLock<Box<[IVec3]>> = LazyLock::new(|| {
    GENERATING_SECTIONS
        .iter()
        .copied()
        .flat_map(|position| {
            iter::repeat(position).zip(-VERTICAL_RENDER_DISTANCE..VERTICAL_RENDER_DISTANCE + 1)
        })
        .map(|(position, y)| position.with_y(y))
        .collect()
});

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, camera: &Camera, context: &Context) {
        let origin = camera.transformation().position().as_ivec3() / Chunk::SIZE as i32;

        let generated_positions = {
            GENERATING_SECTIONS
                .iter()
                .copied()
                .filter_map(|position| {
                    let position =
                        ChunkSectionPosition::new(position.x + origin.x, position.z + origin.z);
                    (!self.generated_sections.contains(&position)).then_some(position)
                })
                .collect::<BTreeSet<_>>()
        };
        if !generated_positions.is_empty() {
            self.generate_sections(&generated_positions);

            let sections_positions = generated_positions
                .par_iter()
                .copied()
                .flat_map_iter(|position| position.adjacent())
                .filter(|position| self.generated_sections.contains(position))
                .filter(|position| !generated_positions.contains(&position))
                .collect::<BTreeSet<_>>();
            self.update_sections_meshes(sections_positions, context);
        }

        {
            let positions = RENDERING_CHUNKS_OFFSETS
                .iter()
                .copied()
                .map(|position| position + origin)
                .filter(|position| !self.meshes.contains_key(position));
            let meshes = positions
                .par_bridge()
                .map(|position| (position, self.make_mesh(position, context)))
                .collect::<Vec<_>>();
            self.meshes.extend(meshes);
        }
    }

    pub fn get(&self, position: IVec3) -> &Chunk {
        self.chunks.get(&position).unwrap_or(&EMPTY_CHUNK)
    }

    pub fn generate_sections(&mut self, positions: &BTreeSet<ChunkSectionPosition>) {
        let chunks = positions
            .par_iter()
            .copied()
            .flat_map_iter(|position| {
                let section = self.generator.generate_section(position);
                section
                    .chunks
                    .into_iter()
                    .enumerate()
                    .map(move |(y, chunk)| (position.with_y(y as i32), chunk))
            })
            .collect::<Vec<(IVec3, Chunk)>>();

        self.chunks.extend(chunks);
        self.generated_sections.extend(positions);
    }

    pub fn update_meshes(
        &mut self,
        chunk_positions: impl ParallelIterator<Item = IVec3>,
        context: &Context,
    ) {
        let meshes = chunk_positions
            .map(|position| (position, self.make_mesh(position, context)))
            .collect::<Vec<_>>();
        self.meshes.extend(meshes);
    }

    pub fn update_sections_meshes(
        &mut self,
        section_positions: impl IntoParallelIterator<Item = ChunkSectionPosition>,
        context: &Context,
    ) {
        let chunk_positions = section_positions
            .into_par_iter()
            .flat_map_iter(|section_position| {
                (0..SECTION_SIZE as i32).map(move |y| section_position.with_y(y))
            });
        self.update_meshes(chunk_positions, context);
    }

    pub fn make_mesh(&self, position: IVec3, context: &Context) -> ChunkBuffer {
        let neighborhood = ChunkNeighborhood::new(self, position);
        let mesh = create_mesh(neighborhood);
        let mesh = ChunkBuffer::from_mesh(&mesh, position, context);
        mesh
    }
}
