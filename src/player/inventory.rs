use bevy::prelude::*;

use crate::world::registry::BlockType;

const HOTBAR_TYPES: [BlockType; 8] = [
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
    counts: [u32; 8],
}

impl Inventory {
    pub fn new() -> Self {
        // Start with a few of each block for testing
        Self {
            selected_slot: 0,
            counts: [64; 8],
        }
    }

    pub fn selected_block(&self) -> BlockType {
        HOTBAR_TYPES[self.selected_slot % HOTBAR_TYPES.len()]
    }

    pub fn count(&self, block: BlockType) -> u32 {
        for (i, t) in HOTBAR_TYPES.iter().enumerate() {
            if *t == block {
                return self.counts[i];
            }
        }
        0
    }

    pub fn add(&mut self, block: BlockType) {
        for (i, t) in HOTBAR_TYPES.iter().enumerate() {
            if *t == block {
                self.counts[i] = self.counts[i].saturating_add(1);
                return;
            }
        }
    }

    pub fn remove(&mut self, block: BlockType) {
        for (i, t) in HOTBAR_TYPES.iter().enumerate() {
            if *t == block {
                self.counts[i] = self.counts[i].saturating_sub(1);
                return;
            }
        }
    }

    pub fn slot_count(&self, slot: usize) -> u32 {
        self.counts[slot % HOTBAR_TYPES.len()]
    }

    pub fn scroll(&mut self, delta: f32) {
        let len = HOTBAR_TYPES.len();
        if delta > 0.0 {
            self.selected_slot = (self.selected_slot + len - 1) % len;
        } else if delta < 0.0 {
            self.selected_slot = (self.selected_slot + 1) % len;
        }
    }
}
