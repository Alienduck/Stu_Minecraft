use shared::block::{BlockType, HOTBAR_TYPES};

#[derive(bevy::prelude::Component)]
pub struct Inventory {
    pub selected_slot: usize,
    counts: [u32; 8],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            selected_slot: 0,
            counts: [64; 8],
        }
    }

    pub fn selected_block(&self) -> BlockType {
        HOTBAR_TYPES[self.selected_slot % HOTBAR_TYPES.len()]
    }

    pub fn count(&self, block: BlockType) -> u32 {
        HOTBAR_TYPES
            .iter()
            .enumerate()
            .find(|(_, t)| **t == block)
            .map(|(i, _)| self.counts[i])
            .unwrap_or(0)
    }

    pub fn add(&mut self, block: BlockType) {
        if let Some(i) = HOTBAR_TYPES.iter().position(|t| *t == block) {
            self.counts[i] = self.counts[i].saturating_add(1);
        }
    }

    pub fn remove(&mut self, block: BlockType) {
        if let Some(i) = HOTBAR_TYPES.iter().position(|t| *t == block) {
            self.counts[i] = self.counts[i].saturating_sub(1);
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
