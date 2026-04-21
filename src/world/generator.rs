use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};

use super::{
    chunk::{CHUNK_HEIGHT, CHUNK_SIZE, Chunk, ChunkCoord},
    registry::BlockType,
};

#[derive(Resource)]
pub struct TerrainGenerator {
    perlin: Perlin,
    seed: u32,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            seed,
        }
    }

    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new();

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = coord.x * CHUNK_SIZE as i32 + x as i32;
                let world_z = coord.z * CHUNK_SIZE as i32 + z as i32;

                let surface_y = self.surface_height(world_x, world_z);
                let tree_chance = self.noise2d(world_x as f64 * 0.3, world_z as f64 * 0.3, 2.0);

                for y in 0..CHUNK_HEIGHT {
                    let block = self.block_at(y, surface_y);
                    chunk.set(x, y, z, block);
                }

                if tree_chance > 0.85 && surface_y + 6 < CHUNK_HEIGHT {
                    self.place_tree(&mut chunk, x, surface_y, z);
                }
            }
        }

        chunk
    }

    fn surface_height(&self, world_x: i32, world_z: i32) -> usize {
        let base = self.noise2d(world_x as f64 * 0.008, world_z as f64 * 0.008, 1.0);
        let detail = self.noise2d(world_x as f64 * 0.04, world_z as f64 * 0.04, 1.0);
        let fine = self.noise2d(world_x as f64 * 0.1, world_z as f64 * 0.1, 1.0);

        let combined = base * 0.6 + detail * 0.3 + fine * 0.1;
        let height = 48 + (combined * 32.0) as i32;
        height.clamp(1, CHUNK_HEIGHT as i32 - 1) as usize
    }

    fn block_at(&self, y: usize, surface_y: usize) -> BlockType {
        if y == 0 {
            return BlockType::Stone;
        }
        if y > surface_y {
            return BlockType::Air;
        }
        if y == surface_y {
            if surface_y < 52 {
                return BlockType::Sand;
            }
            return BlockType::Grass;
        }
        if y >= surface_y.saturating_sub(3) {
            if surface_y < 52 {
                return BlockType::Sand;
            }
            return BlockType::Dirt;
        }
        BlockType::Stone
    }

    fn place_tree(&self, chunk: &mut Chunk, x: usize, surface_y: usize, z: usize) {
        let trunk_height = 4;

        for ty in 1..=trunk_height {
            if surface_y + ty < CHUNK_HEIGHT {
                chunk.set(x, surface_y + ty, z, BlockType::Wood);
            }
        }

        let top = surface_y + trunk_height;
        for ly in 0..3usize {
            let radius: i32 = if ly == 2 { 1 } else { 2 };
            for lx in -radius..=radius {
                for lz in -radius..=radius {
                    let bx = x as i32 + lx;
                    let by = top as i32 + ly as i32;
                    let bz = z as i32 + lz;
                    if Chunk::in_bounds(bx, by, bz) {
                        let bu = bx as usize;
                        let byu = by as usize;
                        let bzu = bz as usize;
                        if chunk.get(bu, byu, bzu) == BlockType::Air {
                            chunk.set(bu, byu, bzu, BlockType::Leaves);
                        }
                    }
                }
            }
        }
    }

    fn noise2d(&self, x: f64, z: f64, scale: f64) -> f64 {
        (self.perlin.get([x * scale, z * scale]) + 1.0) * 0.5
    }
}
