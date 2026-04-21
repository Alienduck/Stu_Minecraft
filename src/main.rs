use bevy::prelude::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    #[default]
    Air,
    Grass,
    Dirt,
    Stone,
}

const CHUNK_SIZE: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;

#[derive(Component)]
struct Chunk {
    data: Box<[BlockType; CHUNK_VOLUME]>,
}

impl Chunk {
    fn new() -> Self {
        Self {
            data: Box::new([BlockType::Air; CHUNK_VOLUME]),
        }
    }
    #[inline(always)]
    fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.data[x + (z * CHUNK_SIZE) + (y * CHUNK_SIZE * CHUNK_SIZE)]
    }
    #[inline(always)]
    fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.data[x + (z * CHUNK_SIZE) + (y * CHUNK_SIZE * CHUNK_SIZE)] = block;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera::default(),
        Transform::from_xyz(0.0, 150.0, 20.0).looking_at(Vec3::new(0.0, 100.0, 0.0), Vec3::Y),
        Player,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI / 4., -PI / 4.)),
    ));
}
