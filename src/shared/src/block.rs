use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlockType {
    #[default]
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Wood = 5,
    Leaves = 6,
    Water = 7,
}

impl BlockType {
    // If you dont know about inline in Rust, must see -> https://matklad.github.io/2021/07/09/inline-in-rust.html
    #[inline(always)]
    pub fn is_solid(self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water)
    }

    #[inline(always)]
    pub fn is_opaque(self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water | BlockType::Leaves)
    }

    /// Break time in seconds. 0.0 = unbreakable.
    pub fn break_time(self) -> f32 {
        match self {
            BlockType::Air => 0.0,
            BlockType::Grass => 0.6,
            BlockType::Dirt => 0.6,
            BlockType::Sand => 0.6,
            BlockType::Stone => 1.5,
            BlockType::Wood => 1.2,
            BlockType::Leaves => 0.3,
            // TODO: Should make it unpickable
            BlockType::Water => f32::MAX,
        }
    }

    /// Linear-light RGBA used by the vertex-color mesh.
    pub fn linear_color(self) -> [f32; 4] {
        match self {
            BlockType::Air => [0.00, 0.00, 0.00, 0.0],
            BlockType::Grass => [0.07, 0.36, 0.04, 1.0],
            BlockType::Dirt => [0.25, 0.12, 0.03, 1.0],
            BlockType::Stone => [0.20, 0.20, 0.20, 1.0],
            BlockType::Sand => [0.84, 0.73, 0.33, 1.0],
            BlockType::Wood => [0.17, 0.07, 0.01, 1.0],
            BlockType::Leaves => [0.01, 0.24, 0.01, 1.0],
            BlockType::Water => [0.03, 0.13, 0.56, 0.7],
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => BlockType::Grass,
            2 => BlockType::Dirt,
            3 => BlockType::Stone,
            4 => BlockType::Sand,
            5 => BlockType::Wood,
            6 => BlockType::Leaves,
            7 => BlockType::Water,
            _ => BlockType::Air,
        }
    }
}

pub const HOTBAR_TYPES: [BlockType; 8] = [
    BlockType::Grass,
    BlockType::Dirt,
    BlockType::Stone,
    BlockType::Sand,
    BlockType::Wood,
    BlockType::Leaves,
    BlockType::Water,
    BlockType::Stone,
];
