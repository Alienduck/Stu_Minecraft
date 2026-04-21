use bevy::prelude::*;

use crate::world::registry::BlockType;

const HOTBAR: [BlockType; 8] = [
    BlockType::Grass,
    BlockType::Dirt,
    BlockType::Stone,
    BlockType::Sand,
    BlockType::Wood,
    BlockType::Leaves,
    BlockType::Water,
    BlockType::Stone,
];

#[derive(Component)]
pub struct Inventory {
    pub selected_slot: usize,
}

impl Inventory {
    pub fn new() -> Self {
        Self { selected_slot: 0 }
    }

    pub fn selected_block(&self) -> BlockType {
        HOTBAR[self.selected_slot % HOTBAR.len()]
    }

    pub fn scroll(&mut self, delta: f32) {
        let len = HOTBAR.len();
        if delta > 0.0 {
            self.selected_slot = (self.selected_slot + len - 1) % len;
        } else if delta < 0.0 {
            self.selected_slot = (self.selected_slot + 1) % len;
        }
    }
}
