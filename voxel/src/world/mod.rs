pub mod block;
pub mod chunk;
pub mod face;
pub mod generator;
pub mod meshes;

pub use block::{Block, Visibility};
use chunk::{Chunk, ChunkSectionPosition, CHUNK_SIZE};
pub use face::{Direction, Face};
use generator::{DefaultGenerator, Generate};
use glam::IVec3;
pub use meshes::RawMesh;
use std::iter;

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::application::MeshGenerator;
use crate::camera::Camera;

const HORIZONTAL_RENDER_DISTANCE: i32 = 16;
const VERTICAL_RENDER_DISTANCE: i32 = 10;
const GENERATION_DISTANCE: i32 = HORIZONTAL_RENDER_DISTANCE + 1;

static GENERATING_SECTIONS_OFFSETS: LazyLock<Box<[ChunkSectionPosition]>> = LazyLock::new(|| {
    let mut res = (-GENERATION_DISTANCE..=GENERATION_DISTANCE)
        .flat_map(|x| iter::repeat(x).zip(-GENERATION_DISTANCE..=GENERATION_DISTANCE))
        .map(ChunkSectionPosition::from)
        .collect::<Box<_>>();
    res.sort_by_key(|position| position.x.pow(2) + position.z.pow(2));
    res
});

static VISIBLE_CHUNKS_OFFSETS: LazyLock<Box<[IVec3]>> = LazyLock::new(|| {
    let mut res = (-HORIZONTAL_RENDER_DISTANCE..=HORIZONTAL_RENDER_DISTANCE)
        .flat_map(|x| iter::repeat(x).zip(-HORIZONTAL_RENDER_DISTANCE..=HORIZONTAL_RENDER_DISTANCE))
        .flat_map(|position| {
            iter::repeat(position).zip(-VERTICAL_RENDER_DISTANCE..=VERTICAL_RENDER_DISTANCE)
        })
        .map(|((x, z), y)| IVec3::new(x, y, z))
        .collect::<Box<_>>();
    res.sort_by_key(|position| position.length_squared());
    res
});

#[derive(Default)]
pub struct World {
    chunks: HashMap<IVec3, Chunk>,
    generated_sections: HashSet<ChunkSectionPosition>,
    generator: DefaultGenerator,
    previous_origin: IVec3,
}

impl World {
    pub fn update(&mut self, camera: &Camera, mesh_generator: &MeshGenerator) {
        let origin = camera.transformation().position().as_ivec3() / CHUNK_SIZE as i32;
        if origin == self.previous_origin {
            return;
        }
        self.previous_origin = origin;

        self.update_chunks(origin, mesh_generator);
        self.update_visible_chunks(origin, mesh_generator);
    }

    fn update_chunks(&mut self, origin: IVec3, mesh_generator: &MeshGenerator) {
        let origin = origin.into();
        let new_sections_positions = {
            GENERATING_SECTIONS_OFFSETS
                .iter()
                .copied()
                .map(|position| position + origin)
                .filter(|&position| self.generated_sections.insert(position))
        };

        let new_chunks = new_sections_positions
            .flat_map(|position| {
                let section = self.generator.generate_section(position);
                section
                    .into_chunks()
                    .map(move |(y, chunk)| (position.with_y(y as i32), chunk))
            })
            .collect::<Box<_>>();
        if new_chunks.is_empty() {
            return;
        }

        self.chunks.extend(new_chunks.iter().cloned());
        mesh_generator.insert_chunks(new_chunks);
    }

    fn update_visible_chunks(&self, origin: IVec3, mesh_generator: &MeshGenerator) {
        let visible_chunks = VISIBLE_CHUNKS_OFFSETS
            .iter()
            .copied()
            .map(|position| position + origin)
            .filter(|position| self.chunks.contains_key(position))
            .collect::<Box<_>>();
        mesh_generator.set_visible(visible_chunks);
    }
}
