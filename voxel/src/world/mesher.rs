use glam::{uvec3, UVec3};

use crate::render::Vertex;

use super::{
    block::Visibility,
    block_face::{BlockFace, Direction},
    chunk_map::ChunkBoundary,
};

#[derive(Debug, Default, Clone)]
pub struct Mesh {
    verticies: Vec<Vertex>,
    indices: Vec<u16>,

    offset: u16,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_face(&mut self, block_face: BlockFace) {
        self.verticies.extend(block_face.vertices());
        self.indices.extend(BlockFace::indices(self.offset));
        self.offset += 1;
    }

    pub fn verticies(&self) -> &[Vertex] {
        &self.verticies
    }

    pub fn indices(&self) -> &[u16] {
        &self.indices
    }
}

pub trait Mesher {
    fn mesh(chunk: &ChunkBoundary, mesh: &mut Mesh);
}

pub struct CulledMesher;

impl CulledMesher {
    fn ao_value(side1: bool, corner: bool, side2: bool) -> u32 {
        match (side1, corner, side2) {
            (true, _, true) => 0,
            (true, true, false) | (false, true, true) => 1,
            (false, false, false) => 3,
            _ => 2,
        }
    }

    fn ao_values(chunk: &ChunkBoundary, position: UVec3, direction: Direction) -> [u32; 4] {
        let UVec3 { x, y, z } = position;

        let neighbors = match direction {
            Direction::Left => [
                chunk[(x - 1, y, z - 1)],
                chunk[(x - 1, y + 1, z - 1)],
                chunk[(x - 1, y + 1, z)],
                chunk[(x - 1, y + 1, z + 1)],
                chunk[(x - 1, y, z + 1)],
                chunk[(x - 1, y.saturating_sub(1), z + 1)],
                chunk[(x - 1, y.saturating_sub(1), z)],
                chunk[(x - 1, y.saturating_sub(1), z - 1)],
            ],
            Direction::Right => [
                chunk[(x + 1, y, z - 1)],
                chunk[(x + 1, y + 1, z - 1)],
                chunk[(x + 1, y + 1, z)],
                chunk[(x + 1, y + 1, z + 1)],
                chunk[(x + 1, y, z + 1)],
                chunk[(x + 1, y.saturating_sub(1), z + 1)],
                chunk[(x + 1, y.saturating_sub(1), z)],
                chunk[(x + 1, y.saturating_sub(1), z - 1)],
            ],
            Direction::Bottom => [
                chunk[(x - 1, y.saturating_sub(1), z)],
                chunk[(x - 1, y.saturating_sub(1), z - 1)],
                chunk[(x, y.saturating_sub(1), z - 1)],
                chunk[(x + 1, y.saturating_sub(1), z - 1)],
                chunk[(x + 1, y.saturating_sub(1), z)],
                chunk[(x + 1, y.saturating_sub(1), z + 1)],
                chunk[(x, y.saturating_sub(1), z + 1)],
                chunk[(x - 1, y.saturating_sub(1), z + 1)],
            ],
            Direction::Top => [
                chunk[(x - 1, y + 1, z)],
                chunk[(x - 1, y + 1, z - 1)],
                chunk[(x, y + 1, z - 1)],
                chunk[(x + 1, y + 1, z - 1)],
                chunk[(x + 1, y + 1, z)],
                chunk[(x + 1, y + 1, z + 1)],
                chunk[(x, y + 1, z + 1)],
                chunk[(x - 1, y + 1, z + 1)],
            ],
            Direction::Back => [
                chunk[(x - 1, y, z - 1)],
                chunk[(x - 1, y + 1, z - 1)],
                chunk[(x, y + 1, z - 1)],
                chunk[(x + 1, y + 1, z - 1)],
                chunk[(x + 1, y, z - 1)],
                chunk[(x + 1, y.saturating_sub(1), z - 1)],
                chunk[(x, y.saturating_sub(1), z - 1)],
                chunk[(x - 1, y.saturating_sub(1), z - 1)],
            ],
            Direction::Front => [
                chunk[(x - 1, y, z + 1)],
                chunk[(x - 1, y + 1, z + 1)],
                chunk[(x, y + 1, z + 1)],
                chunk[(x + 1, y + 1, z + 1)],
                chunk[(x + 1, y, z + 1)],
                chunk[(x + 1, y.saturating_sub(1), z + 1)],
                chunk[(x, y.saturating_sub(1), z + 1)],
                chunk[(x - 1, y.saturating_sub(1), z + 1)],
            ],
        };

        let neighbors = neighbors.map(|neighbor| neighbor.is_opaque());
        [
            CulledMesher::ao_value(neighbors[0], neighbors[1], neighbors[2]),
            CulledMesher::ao_value(neighbors[2], neighbors[3], neighbors[4]),
            CulledMesher::ao_value(neighbors[4], neighbors[5], neighbors[6]),
            CulledMesher::ao_value(neighbors[6], neighbors[7], neighbors[0]),
        ]
    }
}

impl Mesher for CulledMesher {
    fn mesh(chunk: &ChunkBoundary, mesh: &mut Mesh) {
        for (x, y, z) in ChunkBoundary::meshing_range() {
            let position = uvec3(x, y, z);

            if (x > 0 && x < 18 - 1) && (y > 0 && y < 18 - 1) && (z > 0 && z < 18 - 1) {
                let current = chunk[position];
                if current.is_empty() {
                    continue;
                }
                let neighbors = [
                    Direction::Bottom,
                    Direction::Top,
                    Direction::Left,
                    Direction::Right,
                    Direction::Front,
                    Direction::Back,
                ];

                for direction in neighbors {
                    let neighbor = position.saturating_add_signed(direction.to_vec());
                    let neighbor = chunk[neighbor];

                    let should_generate = match (current.visibility(), neighbor.visibility()) {
                        (Visibility::Opaque, Visibility::Empty)
                        | (Visibility::Opaque, Visibility::Transparent)
                        | (Visibility::Transparent, Visibility::Empty) => true,

                        (Visibility::Transparent, Visibility::Transparent) => current != neighbor,
                        (_, _) => false,
                    };

                    let ao_value = Self::ao_values(&chunk, position, direction);

                    if should_generate {
                        mesh.push_face(BlockFace::new(current, position, ao_value, direction))
                    }
                }
            }
        }
    }
}
