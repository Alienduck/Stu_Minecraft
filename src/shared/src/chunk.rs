// src/shared/src/chunk.rs

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

    /// Returns Air for out-of-bounds.
    #[inline(always)]
    pub fn get_safe(&self, x: i32, y: i32, z: i32) -> BlockType {
        if Self::in_bounds(x, y, z) {
            self.get(x as usize, y as usize, z as usize)
        } else {
            BlockType::Air
        }
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

// ── Face table ────────────────────────────────────────────────────────────────

/// (normal, vertices[4])  CCW winding from outside.
const FACES: [([i32; 3], [[f32; 3]; 4]); 6] = [
    // 0 +Y top
    (
        [0, 1, 0],
        [[0., 1., 1.], [1., 1., 1.], [1., 1., 0.], [0., 1., 0.]],
    ),
    // 1 -Y bottom
    (
        [0, -1, 0],
        [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]],
    ),
    // 2 +X right
    (
        [1, 0, 0],
        [[1., 0., 1.], [1., 0., 0.], [1., 1., 0.], [1., 1., 1.]],
    ),
    // 3 -X left
    (
        [-1, 0, 0],
        [[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]],
    ),
    // 4 +Z front
    (
        [0, 0, 1],
        [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
    ),
    // 5 -Z back
    (
        [0, 0, -1],
        [[1., 0., 0.], [0., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
    ),
];

const FACE_BRIGHTNESS: [f32; 6] = [1.0, 0.5, 0.8, 0.8, 0.9, 0.7];

// ── Ambient Occlusion ─────────────────────────────────────────────────────────

/// Classic smooth vertex AO.
/// `side1`, `side2`: edge-adjacent block is solid?
/// `corner`: diagonal block is solid?
/// Returns brightness multiplier 1.0 → 0.25.
#[inline(always)]
fn vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    let level = if side1 && side2 {
        3
    } else {
        side1 as u8 + side2 as u8 + corner as u8
    };
    match level {
        0 => 1.0,
        1 => 0.75,
        2 => 0.50,
        _ => 0.25,
    }
}

/// Compute per-vertex AO for the 4 vertices of a face.
///
/// The face is defined by its block position (bx,by,bz) and the normal direction.
/// For each vertex we sample 2 side neighbours + 1 corner on the normal-offset layer.
///
/// Tangent axes are derived from the normal so there is no per-face hard-coding.
/// Vertex order matches the FACES table (CCW from outside):
///   vi 0 = (−t1, −t2),  vi 1 = (+t1, −t2)
///   vi 2 = (+t1, +t2),  vi 3 = (−t1, +t2)
fn face_ao(chunk: &Chunk, bx: i32, by: i32, bz: i32, normal: [i32; 3]) -> [f32; 4] {
    let [nx, ny, nz] = normal;

    // Two tangent axes orthogonal to the normal
    let (t1, t2) = if nx != 0 {
        ([0_i32, 1, 0], [0_i32, 0, 1])
    } else if ny != 0 {
        ([1_i32, 0, 0], [0_i32, 0, 1])
    } else {
        ([1_i32, 0, 0], [0_i32, 1, 0])
    };

    // Sign pairs for the four vertex corners
    let signs: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];

    signs.map(|(s1, s2)| {
        let side1 = chunk
            .get_safe(
                bx + nx + t1[0] * s1,
                by + ny + t1[1] * s1,
                bz + nz + t1[2] * s1,
            )
            .is_opaque();
        let side2 = chunk
            .get_safe(
                bx + nx + t2[0] * s2,
                by + ny + t2[1] * s2,
                bz + nz + t2[2] * s2,
            )
            .is_opaque();
        let corner = chunk
            .get_safe(
                bx + nx + t1[0] * s1 + t2[0] * s2,
                by + ny + t1[1] * s1 + t2[1] * s2,
                bz + nz + t1[2] * s1 + t2[2] * s2,
            )
            .is_opaque();
        vertex_ao(side1, side2, corner)
    })
}

// ── Mesh builder ─────────────────────────────────────────────────────────────

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

                // Air and Water are skipped — water is rendered by the client's
                // WaterMaterial plugin on a separate mesh.
                if block == BlockType::Air || block == BlockType::Water {
                    continue;
                }

                let base_color = block.linear_color();
                let bx = x as i32;
                let by = y as i32;
                let bz = z as i32;

                for (face_idx, (normal, verts)) in FACES.iter().enumerate() {
                    let [nx, ny, nz] = *normal;
                    let nnx = bx + nx;
                    let nny = by + ny;
                    let nnz = bz + nz;

                    // Cull if neighbour is opaque (or same transparent block)
                    if Chunk::in_bounds(nnx, nny, nnz) {
                        let nb = chunk.get(nnx as usize, nny as usize, nnz as usize);
                        if nb.is_opaque() {
                            continue;
                        }
                        if nb == block && !block.is_opaque() {
                            continue;
                        }
                    }

                    let face_bright = FACE_BRIGHTNESS[face_idx];
                    let ao = face_ao(chunk, bx, by, bz, *normal);

                    let base = positions.len() as u32;
                    for (vi, v) in verts.iter().enumerate() {
                        let brightness = face_bright * ao[vi];
                        positions.push([v[0] + x as f32, v[1] + y as f32, v[2] + z as f32]);
                        normals.push([nx as f32, ny as f32, nz as f32]);
                        uvs.push([v[0], v[1]]);
                        colors.push([
                            base_color[0] * brightness,
                            base_color[1] * brightness,
                            base_color[2] * brightness,
                            base_color[3],
                        ]);
                    }

                    // AO-correct diagonal flip: avoids seam on concave corners
                    let b = base;
                    if ao[0] + ao[2] < ao[1] + ao[3] {
                        indices.extend_from_slice(&[b, b + 1, b + 3, b + 1, b + 2, b + 3]);
                    } else {
                        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
                    }
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
