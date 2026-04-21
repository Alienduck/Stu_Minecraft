use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BlockType {
    #[default]
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Wood,
    Leaves,
    Water,
}

impl BlockType {
    pub fn is_solid(self) -> bool {
        self != BlockType::Air
    }
}

#[derive(Resource)]
pub struct BlockRegistry;

impl BlockRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn color(&self, block: BlockType) -> [f32; 4] {
        match block {
            BlockType::Air => [0.0, 0.0, 0.0, 0.0],
            BlockType::Grass => [0.29, 0.65, 0.24, 1.0],
            BlockType::Dirt => [0.55, 0.38, 0.21, 1.0],
            BlockType::Stone => [0.50, 0.50, 0.50, 1.0],
            BlockType::Sand => [0.93, 0.87, 0.62, 1.0],
            BlockType::Wood => [0.45, 0.30, 0.15, 1.0],
            BlockType::Leaves => [0.13, 0.55, 0.13, 1.0],
            BlockType::Water => [0.20, 0.40, 0.80, 0.7],
        }
    }
}
