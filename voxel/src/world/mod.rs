pub mod block;
pub mod chunk;
pub mod face;
pub mod generator;
pub mod meshes;

pub use block::{Block, Visibility};
pub use chunk::Chunk;
use chunk::{ChunkSectionPosition, Volume};
pub use face::{Direction, Face};
use generator::{DefaultGenerator, Generate};
use meshes::MeshesMessage;
pub use meshes::RawMesh;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::iter;
use std::sync::mpsc::Sender;

use std::collections::{BTreeSet, HashSet};
use std::sync::LazyLock;

use glam::IVec3;

use crate::camera::Camera;

const HORIZONTAL_RENDER_DISTANCE: i32 = 12;
const VERTICAL_RENDER_DISTANCE: i32 = 12;
const GENERATION_DISTANCE: i32 = HORIZONTAL_RENDER_DISTANCE + 1;

pub static EMPTY_CHUNK: LazyLock<Chunk> = LazyLock::new(|| Chunk::default());

mod chunks {
    use std::{collections::HashMap, ops::Deref, sync::Arc};

    use glam::IVec3;
    use parking_lot::{RwLock, RwLockReadGuard};

    use super::Chunk;

    pub type ChunksInner = HashMap<IVec3, Chunk>;

    #[derive(Debug, Default, Clone)]
    pub struct Chunks(Arc<RwLock<ChunksInner>>);

    impl Chunks {
        pub fn read(&self) -> ChunksReadGuard<'_> {
            ChunksReadGuard(self.0.read())
        }

        pub fn extend(&self, iter: impl IntoIterator<Item = (IVec3, Chunk)>) {
            self.0.write().extend(iter);
        }
    }

    pub struct ChunksReadGuard<'s>(RwLockReadGuard<'s, ChunksInner>);

    impl Deref for ChunksReadGuard<'_> {
        type Target = HashMap<IVec3, Chunk>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
pub use chunks::*;

#[derive(Default)]
pub struct World {
    pub chunks: Chunks,
    generated_sections: HashSet<ChunkSectionPosition>,
    generator: DefaultGenerator,
    generated_meshes: HashSet<IVec3>,
}

impl World {
    pub fn update(
        &mut self,
        camera: &Camera,
        // TODO group into `UpdateSender`
        position_sender: &Sender<IVec3>,
        meshes_sender: &Sender<MeshesMessage>,
    ) {
        let origin = camera.transformation().position().as_ivec3() / Chunk::SIZE as i32;

        let generated_section_positions = {
            (-GENERATION_DISTANCE..=GENERATION_DISTANCE)
                .flat_map(|x| iter::repeat(x).zip(-GENERATION_DISTANCE..=GENERATION_DISTANCE))
                .map(|(x, z)| ChunkSectionPosition::new(x, z))
                .filter_map(|position| {
                    let position =
                        ChunkSectionPosition::new(position.x + origin.x, position.z + origin.z);
                    (!self.generated_sections.contains(&position)).then_some(position)
                })
                .collect::<BTreeSet<_>>()
        };
        self.generate_sections(&generated_section_positions);

        let visible_sections_positions = (-HORIZONTAL_RENDER_DISTANCE..=HORIZONTAL_RENDER_DISTANCE)
            .flat_map(|x| {
                iter::repeat(x).zip(-HORIZONTAL_RENDER_DISTANCE..=HORIZONTAL_RENDER_DISTANCE)
            })
            .map(|(x, z)| ChunkSectionPosition::new(x, z));

        {
            let visible_chunks = {
                let chunks = self.chunks.read();
                visible_sections_positions
                    .flat_map(|position| {
                        iter::repeat(position)
                            .zip(-VERTICAL_RENDER_DISTANCE..=VERTICAL_RENDER_DISTANCE)
                    })
                    .map(|(position, y)| position.with_y(y))
                    .map(move |position| position + origin)
                    .filter(|position| chunks.get(position).is_some())
                    .collect::<Vec<_>>()
            };

            let mut non_generated_visible_chunks = visible_chunks
                .iter()
                .copied()
                .filter(|position| !self.generated_meshes.contains(position))
                .collect::<Vec<_>>();
            non_generated_visible_chunks
                .sort_unstable_by_key(|&position| (position - origin).length_squared());

            self.generated_meshes.extend(&non_generated_visible_chunks);
            for position in non_generated_visible_chunks {
                position_sender.send(position).unwrap();
            }

            let mut ungenerated_meshes = Vec::default();
            self.generated_meshes.retain(|&position| {
                if visible_chunks.contains(&position) {
                    true
                } else {
                    ungenerated_meshes.push(position);
                    false
                }
            });
            for position in ungenerated_meshes {
                meshes_sender
                    .send(MeshesMessage::Ungenerate { position })
                    .unwrap();
            }
        }
    }

    pub fn generate_sections(&mut self, positions: &BTreeSet<ChunkSectionPosition>) {
        if positions.is_empty() {
            return;
        }

        let chunks = positions
            .par_iter()
            .copied()
            .flat_map_iter(|position| {
                let section = self.generator.generate_section(position);
                section
                    .into_chunks()
                    .map(move |(y, chunk)| (position.with_y(y as i32), chunk))
            })
            .collect::<Vec<_>>();

        self.chunks.extend(chunks);
        self.generated_sections.extend(positions);
    }
}
