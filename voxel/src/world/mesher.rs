use std::iter;

use glam::{ivec3, uvec3, UVec3};

use crate::{render::Vertex, world::chunk::CHUNK_SIZE};

use super::{chunk::ChunkNeighborhood, face::Face, Direction, Visibility};

#[derive(Debug, Default, Clone)]
pub struct RawMesh {
    verticies: Vec<Vertex>,
    indices: Vec<u16>,
    offset: u16,
}

impl RawMesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn verticies(&self) -> &[Vertex] {
        &self.verticies
    }

    pub fn push_face(&mut self, block_face: Face) {
        self.verticies.extend(block_face.vertices());
        self.indices.extend(Face::indices(self.offset));
        self.offset += 1;
    }

    pub fn indices(&self) -> &[u16] {
        &self.indices
    }
}

// Making this `static` does not give any effect
const NEIGHBORS: [Direction; 6] = [
    Direction::Bottom,
    Direction::Top,
    Direction::Left,
    Direction::Right,
    Direction::Front,
    Direction::Back,
];

pub fn create_mesh(neighborhood: ChunkNeighborhood) -> RawMesh {
    // Making this `static` does not give any effect
    let meshing_range = (1..CHUNK_SIZE as u32 + 1)
        .flat_map(move |i| iter::repeat(i).zip(1..CHUNK_SIZE as u32 + 1))
        .flat_map(move |i| iter::repeat(i).zip(1..CHUNK_SIZE as u32 + 1))
        .map(|((x, y), z)| uvec3(x, y, z));

    let block_faces = meshing_range
        .map(|position| (position, neighborhood[position]))
        .filter(|&(_, current)| current.visibility() != Visibility::Empty)
        .flat_map(|(position, current)| {
            NEIGHBORS.into_iter().filter_map(move |direction| {
                let neighbor = position.wrapping_add_signed(direction.to_vec());
                let neighbor = neighborhood[neighbor];
                if neighbor.visibility() == Visibility::Opaque || neighbor == current {
                    return None;
                }

                // May be slow, needs investigation
                let ao = ao_values(neighborhood, position, direction);
                Some(Face::new(current, position, ao, direction))
            })
        });

    let mut mesh = RawMesh::default();
    for block_face in block_faces {
        mesh.push_face(block_face);
    }
    mesh
}

fn ao_value(side1: bool, corner: bool, side2: bool) -> u8 {
    match (side1, corner, side2) {
        (false, false, false) => 3,
        (true, true, false) | (false, true, true) => 1,
        (true, _, true) => 0,
        _ => 2,
    }
}

fn ao_values(neighborhood: ChunkNeighborhood, position: UVec3, direction: Direction) -> [u8; 4] {
    let neighbor_offsets = match direction {
        Direction::Left => [
            ivec3(-1, 0, -1),
            ivec3(-1, 1, -1),
            ivec3(-1, 1, 0),
            ivec3(-1, 1, 1),
            ivec3(-1, 0, 1),
            ivec3(-1, -1, 1),
            ivec3(-1, -1, 0),
            ivec3(-1, -1, -1),
        ],
        Direction::Right => [
            ivec3(1, 0, 1),
            ivec3(1, 1, 1),
            ivec3(1, 1, 0),
            ivec3(1, 1, -1),
            ivec3(1, 0, -1),
            ivec3(1, -1, -1),
            ivec3(1, -1, 0),
            ivec3(1, -1, 1),
        ],
        Direction::Bottom => [
            ivec3(-1, -1, 0),
            ivec3(-1, -1, -1),
            ivec3(0, -1, -1),
            ivec3(1, -1, -1),
            ivec3(1, -1, 0),
            ivec3(1, -1, 1),
            ivec3(0, -1, 1),
            ivec3(-1, -1, 1),
        ],
        Direction::Top => [
            ivec3(-1, 1, 0),
            ivec3(-1, 1, -1),
            ivec3(0, 1, -1),
            ivec3(1, 1, -1),
            ivec3(1, 1, 0),
            ivec3(1, 1, 1),
            ivec3(0, 1, 1),
            ivec3(-1, 1, 1),
        ],
        Direction::Back => [
            ivec3(1, 0, -1),
            ivec3(1, 1, -1),
            ivec3(0, 1, -1),
            ivec3(-1, 1, -1),
            ivec3(-1, 0, -1),
            ivec3(-1, -1, -1),
            ivec3(0, -1, -1),
            ivec3(1, -1, -1),
        ],
        Direction::Front => [
            ivec3(-1, 0, 1),
            ivec3(-1, 1, 1),
            ivec3(0, 1, 1),
            ivec3(1, 1, 1),
            ivec3(1, 0, 1),
            ivec3(1, -1, 1),
            ivec3(0, -1, 1),
            ivec3(-1, -1, 1),
        ],
    };
    let neighbors = neighbor_offsets.map(|offset| {
        let block = neighborhood[position.wrapping_add_signed(offset)];
        block.visibility() == Visibility::Opaque
    });

    [
        ao_value(neighbors[0], neighbors[1], neighbors[2]),
        ao_value(neighbors[2], neighbors[3], neighbors[4]),
        ao_value(neighbors[4], neighbors[5], neighbors[6]),
        ao_value(neighbors[6], neighbors[7], neighbors[0]),
    ]
}
