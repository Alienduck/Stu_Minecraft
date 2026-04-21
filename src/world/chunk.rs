use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::PrimitiveTopology};

use super::registry::{BlockRegistry, BlockType};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 128;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

#[derive(Component)]
pub struct Chunk {
    pub blocks: Box<[BlockType; CHUNK_VOLUME]>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: Box::new([BlockType::Air; CHUNK_VOLUME]),
        }
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks[x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE]
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.blocks[x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE] = block;
    }

    pub fn in_bounds(x: i32, y: i32, z: i32) -> bool {
        x >= 0
            && x < CHUNK_SIZE as i32
            && y >= 0
            && y < CHUNK_HEIGHT as i32
            && z >= 0
            && z < CHUNK_SIZE as i32
    }
}

const FACES: [([f32; 3], [[f32; 3]; 4]); 6] = [
    // +Y top: CCW from above
    (
        [0.0, 1.0, 0.0],
        [
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
    ),
    // -Y bottom: CCW from below
    (
        [0.0, -1.0, 0.0],
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    ),
    // +X right: CCW from right
    (
        [1.0, 0.0, 0.0],
        [
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
    ),
    // -X left: CCW from left
    (
        [-1.0, 0.0, 0.0],
        [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    ),
    // +Z front: CCW from front
    (
        [0.0, 0.0, 1.0],
        [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
    ),
    // -Z back: CCW from back
    (
        [0.0, 0.0, -1.0],
        [
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
    ),
];

// Ambient occlusion-style brightness per face to give depth without textures
const FACE_BRIGHTNESS: [f32; 6] = [
    1.0, // top
    0.5, // bottom
    0.8, // right
    0.8, // left
    0.9, // front
    0.7, // back
];

pub fn build_chunk_mesh(chunk: &Chunk, registry: &BlockRegistry) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);
                if block == BlockType::Air {
                    continue;
                }

                let base_color = registry.linear_color(block);

                for (face_idx, (normal, verts)) in FACES.iter().enumerate() {
                    let nx = x as i32 + normal[0] as i32;
                    let ny = y as i32 + normal[1] as i32;
                    let nz = z as i32 + normal[2] as i32;

                    let neighbor_solid = if Chunk::in_bounds(nx, ny, nz) {
                        chunk.get(nx as usize, ny as usize, nz as usize) != BlockType::Air
                    } else {
                        false
                    };

                    if neighbor_solid {
                        continue;
                    }

                    let brightness = FACE_BRIGHTNESS[face_idx];
                    let color = [
                        base_color[0] * brightness,
                        base_color[1] * brightness,
                        base_color[2] * brightness,
                        base_color[3],
                    ];

                    let base = positions.len() as u32;
                    for v in verts {
                        positions.push([v[0] + x as f32, v[1] + y as f32, v[2] + z as f32]);
                        normals.push(*normal);
                        colors.push(color);
                        uvs.push([0.0, 0.0]);
                    }
                    // CCW winding: 0,1,2 and 0,2,3
                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}
