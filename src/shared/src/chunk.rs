use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::block::BlockType;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 128;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

/// Flat array: index = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
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
    pub fn idx(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks[Self::idx(x, y, z)]
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.blocks[Self::idx(x, y, z)] = block;
    }

    #[inline(always)]
    pub fn in_bounds(x: i32, y: i32, z: i32) -> bool {
        x >= 0
            && x < CHUNK_SIZE as i32
            && y >= 0
            && y < CHUNK_HEIGHT as i32
            && z >= 0
            && z < CHUNK_SIZE as i32
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RawMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

/// Fun Fact, just by reversing 2 faces you aren't capable to see it
/// And I've been troll by this thing at begining of the development
const FACES: [([f32; 3], [[f32; 3]; 4]); 6] = [
    // +Y top
    (
        [0., 1., 0.],
        [[0., 1., 1.], [1., 1., 1.], [1., 1., 0.], [0., 1., 0.]],
    ),
    // -Y bottom
    (
        [0., -1., 0.],
        [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]],
    ),
    // +X right
    (
        [1., 0., 0.],
        [[1., 0., 1.], [1., 0., 0.], [1., 1., 0.], [1., 1., 1.]],
    ),
    // -X left
    (
        [-1., 0., 0.],
        [[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]],
    ),
    // +Z front
    (
        [0., 0., 1.],
        [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
    ),
    // -Z back
    (
        [0., 0., -1.],
        [[1., 0., 0.], [0., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
    ),
];

/// Face brightness: top / bottom / right / left / front / back
const FACE_BRIGHTNESS: [f32; 6] = [1.0, 0.5, 0.8, 0.8, 0.9, 0.7];

pub fn build_chunk_mesh(chunk: &Chunk) -> RawMeshData {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);
                if block == BlockType::Air {
                    continue;
                }
                let base_color = block.linear_color();

                for (face_idx, (normal, verts)) in FACES.iter().enumerate() {
                    let nx = x as i32 + normal[0] as i32;
                    let ny = y as i32 + normal[1] as i32;
                    let nz = z as i32 + normal[2] as i32;

                    // Cull face if neighbour is opaque
                    if Chunk::in_bounds(nx, ny, nz) {
                        let nb = chunk.get(nx as usize, ny as usize, nz as usize);
                        if nb.is_opaque() {
                            continue;
                        }
                        // Same-type transparent blocks don't show interior faces
                        if nb == block && !block.is_opaque() {
                            continue;
                        }
                    }

                    let br = FACE_BRIGHTNESS[face_idx];
                    let color = [
                        base_color[0] * br,
                        base_color[1] * br,
                        base_color[2] * br,
                        base_color[3],
                    ];
                    let base = positions.len() as u32;

                    for v in verts {
                        positions.push([v[0] + x as f32, v[1] + y as f32, v[2] + z as f32]);
                        normals.push(*normal);
                        colors.push(color);
                        uvs.push([0.0, 0.0]);
                    }
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

    RawMeshData {
        positions,
        normals,
        uvs,
        colors,
        indices,
    }
}
