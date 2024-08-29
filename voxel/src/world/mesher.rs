use glam::{uvec3, UVec3};

use crate::render::Vertex;

use super::{chunk::ChunkNeighbors, face::Face, Direction, Visibility};

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

    pub fn push_face(&mut self, block_face: Face) {
        self.verticies.extend(block_face.vertices());
        self.indices.extend(Face::indices(self.offset));
        self.offset += 1;
    }

    pub fn verticies(&self) -> &[Vertex] {
        &self.verticies
    }

    pub fn indices(&self) -> &[u16] {
        &self.indices
    }
}

pub fn create_mesh(chunk: &ChunkNeighbors, mesh: &mut RawMesh) {
    for (x, y, z) in ChunkNeighbors::meshing_range() {
        let position = uvec3(x, y, z);
        let current = chunk[position];

        let neighbors = [
            Direction::Bottom,
            Direction::Top,
            Direction::Left,
            Direction::Right,
            Direction::Front,
            Direction::Back,
        ];

        for direction in neighbors {
            let neighbor = position.wrapping_add_signed(direction.to_vec());
            let neighbor = chunk[neighbor];

            let should_generate = match (current.visibility(), neighbor.visibility()) {
                (Visibility::Opaque, Visibility::Empty)
                | (Visibility::Opaque, Visibility::Transparent)
                | (Visibility::Transparent, Visibility::Empty) => true,

                (Visibility::Transparent, Visibility::Transparent) => current != neighbor,
                (_, _) => false,
            };

            let ao = ao_values(&chunk, position, direction);

            if should_generate {
                mesh.push_face(Face::new(current, position, ao, direction))
            }
        }
    }
}

fn ao_value(side1: bool, corner: bool, side2: bool) -> u8 {
    match (side1, corner, side2) {
        (true, _, true) => 0,
        (true, true, false) | (false, true, true) => 1,
        (false, false, false) => 3,
        _ => 2,
    }
}

fn ao_values(chunk: &ChunkNeighbors, position: UVec3, direction: Direction) -> [u8; 4] {
    let UVec3 { x, y, z } = position;

    let neighbors = match direction {
        Direction::Left => [
            chunk[(x - 1, y, z - 1).into()],
            chunk[(x - 1, y + 1, z - 1).into()],
            chunk[(x - 1, y + 1, z).into()],
            chunk[(x - 1, y + 1, z + 1).into()],
            chunk[(x - 1, y, z + 1).into()],
            chunk[(x - 1, y - 1, z + 1).into()],
            chunk[(x - 1, y - 1, z).into()],
            chunk[(x - 1, y - 1, z - 1).into()],
        ],
        Direction::Right => [
            chunk[(x + 1, y, z + 1).into()],
            chunk[(x + 1, y + 1, z + 1).into()],
            chunk[(x + 1, y + 1, z).into()],
            chunk[(x + 1, y + 1, z - 1).into()],
            chunk[(x + 1, y, z - 1).into()],
            chunk[(x + 1, y - 1, z - 1).into()],
            chunk[(x + 1, y - 1, z).into()],
            chunk[(x + 1, y - 1, z + 1).into()],
        ],
        Direction::Bottom => [
            chunk[(x - 1, y - 1, z).into()],
            chunk[(x - 1, y - 1, z - 1).into()],
            chunk[(x, y - 1, z - 1).into()],
            chunk[(x + 1, y - 1, z - 1).into()],
            chunk[(x + 1, y - 1, z).into()],
            chunk[(x + 1, y - 1, z + 1).into()],
            chunk[(x, y - 1, z + 1).into()],
            chunk[(x - 1, y - 1, z + 1).into()],
        ],
        Direction::Top => [
            chunk[(x - 1, y + 1, z).into()],
            chunk[(x - 1, y + 1, z - 1).into()],
            chunk[(x, y + 1, z - 1).into()],
            chunk[(x + 1, y + 1, z - 1).into()],
            chunk[(x + 1, y + 1, z).into()],
            chunk[(x + 1, y + 1, z + 1).into()],
            chunk[(x, y + 1, z + 1).into()],
            chunk[(x - 1, y + 1, z + 1).into()],
        ],
        Direction::Back => [
            chunk[(x + 1, y, z - 1).into()],
            chunk[(x + 1, y + 1, z - 1).into()],
            chunk[(x, y + 1, z - 1).into()],
            chunk[(x - 1, y + 1, z - 1).into()],
            chunk[(x - 1, y, z - 1).into()],
            chunk[(x - 1, y - 1, z - 1).into()],
            chunk[(x, y - 1, z - 1).into()],
            chunk[(x + 1, y - 1, z - 1).into()],
        ],
        Direction::Front => [
            chunk[(x - 1, y, z + 1).into()],
            chunk[(x - 1, y + 1, z + 1).into()],
            chunk[(x, y + 1, z + 1).into()],
            chunk[(x + 1, y + 1, z + 1).into()],
            chunk[(x + 1, y, z + 1).into()],
            chunk[(x + 1, y - 1, z + 1).into()],
            chunk[(x, y - 1, z + 1).into()],
            chunk[(x - 1, y - 1, z + 1).into()],
        ],
    };

    let neighbors = neighbors.map(|neighbor| matches!(neighbor.visibility(), Visibility::Opaque));
    [
        ao_value(neighbors[0], neighbors[1], neighbors[2]),
        ao_value(neighbors[2], neighbors[3], neighbors[4]),
        ao_value(neighbors[4], neighbors[5], neighbors[6]),
        ao_value(neighbors[6], neighbors[7], neighbors[0]),
    ]
}
