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

    pub fn break_time(self) -> f32 {
        match self {
            BlockType::Air => 0.0,
            BlockType::Grass => 0.6,
            BlockType::Dirt => 0.6,
            BlockType::Sand => 0.6,
            BlockType::Stone => 1.5,
            BlockType::Wood => 1.2,
            BlockType::Leaves => 0.3,
            BlockType::Water => 9999.0,
        }
    }
}

#[derive(Resource)]
pub struct BlockRegistry;

impl BlockRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn linear_color(&self, block: BlockType) -> [f32; 4] {
        match block {
            BlockType::Air => [0.0, 0.0, 0.0, 0.0],
            BlockType::Grass => [0.07, 0.36, 0.04, 1.0],
            BlockType::Dirt => [0.25, 0.12, 0.03, 1.0],
            BlockType::Stone => [0.20, 0.20, 0.20, 1.0],
            BlockType::Sand => [0.84, 0.73, 0.33, 1.0],
            BlockType::Wood => [0.17, 0.07, 0.01, 1.0],
            BlockType::Leaves => [0.01, 0.24, 0.01, 1.0],
            BlockType::Water => [0.03, 0.13, 0.56, 0.7],
        }
    }
}
