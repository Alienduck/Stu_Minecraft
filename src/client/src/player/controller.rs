use bevy::prelude::*;

use shared::{
    block::BlockType,
    chunk::{CHUNK_HEIGHT, CHUNK_SIZE},
};

use crate::{
    input::MovementInput,
    world::{Chunk, ChunkCoordComp},
};

use super::Player;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (apply_gravity, move_player).chain());
    }
}

#[derive(Component, Default)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct Grounded(pub bool);

const GRAVITY: f32 = -28.0;
const JUMP_FORCE: f32 = 9.0;
const WALK_SPEED: f32 = 5.0;
const SPRINT_SPEED: f32 = 9.0;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_WIDTH: f32 = 0.6;

pub fn apply_gravity(time: Res<Time>, mut q: Query<(&mut Velocity, &mut Grounded), With<Player>>) {
    for (mut vel, mut grounded) in q.iter_mut() {
        if !grounded.0 {
            vel.0.y += GRAVITY * time.delta_secs();
        } else if vel.0.y < 0.0 {
            vel.0.y = 0.0;
        }
    }
}

pub fn move_player(
    time: Res<Time>,
    input: Res<MovementInput>,
    chunks: Query<(&Chunk, &ChunkCoordComp)>,
    mut q: Query<(&mut Transform, &mut Velocity, &mut Grounded), With<Player>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut vel, mut grounded) in q.iter_mut() {
        let speed = if input.sprinting {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        let forward = Vec3::new(input.yaw.sin(), 0.0, input.yaw.cos());
        let right = Vec3::new(forward.z, 0.0, -forward.x);

        let mut wish = Vec3::ZERO;
        if input.forward {
            wish -= forward;
        }
        if input.backward {
            wish += forward;
        }
        if input.left {
            wish -= right;
        }
        if input.right {
            wish += right;
        }
        if wish.length_squared() > 0.0 {
            wish = wish.normalize() * speed;
        }

        vel.0.x = wish.x;
        vel.0.z = wish.z;

        if input.jump && grounded.0 {
            vel.0.y = JUMP_FORCE;
            grounded.0 = false;
        }

        let delta = vel.0 * dt;
        let pos = transform.translation;

        let new_x = try_move(pos, Vec3::new(delta.x, 0.0, 0.0), &chunks);
        let new_y = try_move(new_x, Vec3::new(0.0, delta.y, 0.0), &chunks);
        let new_z = try_move(new_y, Vec3::new(0.0, 0.0, delta.z), &chunks);

        grounded.0 = is_on_ground(new_z, &chunks);

        if (new_y - new_x).y.abs() < f32::EPSILON && delta.y != 0.0 {
            vel.0.y = 0.0;
        }

        transform.translation = new_z;
    }
}

fn try_move(pos: Vec3, delta: Vec3, chunks: &Query<(&Chunk, &ChunkCoordComp)>) -> Vec3 {
    let new_pos = pos + delta;
    if !collides_with_world(new_pos, chunks) {
        new_pos
    } else {
        pos
    }
}

/// Guess the collision of the player
fn collides_with_world(pos: Vec3, chunks: &Query<(&Chunk, &ChunkCoordComp)>) -> bool {
    let hw = PLAYER_WIDTH * 0.5;
    let corners = [
        Vec3::new(pos.x - hw, pos.y, pos.z - hw),
        Vec3::new(pos.x + hw, pos.y, pos.z - hw),
        Vec3::new(pos.x - hw, pos.y, pos.z + hw),
        Vec3::new(pos.x + hw, pos.y, pos.z + hw),
        Vec3::new(pos.x - hw, pos.y + PLAYER_HEIGHT, pos.z - hw),
        Vec3::new(pos.x + hw, pos.y + PLAYER_HEIGHT, pos.z - hw),
        Vec3::new(pos.x - hw, pos.y + PLAYER_HEIGHT, pos.z + hw),
        Vec3::new(pos.x + hw, pos.y + PLAYER_HEIGHT, pos.z + hw),
        Vec3::new(pos.x - hw, pos.y + PLAYER_HEIGHT * 0.5, pos.z - hw),
        Vec3::new(pos.x + hw, pos.y + PLAYER_HEIGHT * 0.5, pos.z + hw),
    ];
    corners.iter().any(|c| is_solid_at(*c, chunks))
}

/// Guess if the player is touching the ground
fn is_on_ground(pos: Vec3, chunks: &Query<(&Chunk, &ChunkCoordComp)>) -> bool {
    let hw = PLAYER_WIDTH * 0.5;
    let feet = pos.y - 0.05;
    [
        Vec3::new(pos.x - hw, feet, pos.z - hw),
        Vec3::new(pos.x + hw, feet, pos.z - hw),
        Vec3::new(pos.x - hw, feet, pos.z + hw),
        Vec3::new(pos.x + hw, feet, pos.z + hw),
    ]
    .iter()
    .any(|c| is_solid_at(*c, chunks))
}

/// Guess if the player is in another collider
pub fn is_solid_at(world_pos: Vec3, chunks: &Query<(&Chunk, &ChunkCoordComp)>) -> bool {
    let bx = world_pos.x.floor() as i32;
    let by = world_pos.y.floor() as i32;
    let bz = world_pos.z.floor() as i32;

    // World limit
    if by < 0 {
        return true;
    }

    let cx = bx.div_euclid(CHUNK_SIZE as i32);
    let cz = bz.div_euclid(CHUNK_SIZE as i32);
    let lx = bx.rem_euclid(CHUNK_SIZE as i32) as usize;
    let ly = by as usize;
    let lz = bz.rem_euclid(CHUNK_SIZE as i32) as usize;

    for (chunk, coord_comp) in chunks.iter() {
        if coord_comp.0.x == cx && coord_comp.0.z == cz {
            if ly < CHUNK_HEIGHT {
                return chunk.get(lx, ly, lz).is_solid();
            }
        }
    }
    false
}

/// Public query-compatible block lookup used by input/raycast.
pub fn block_at_world(pos: IVec3, chunks: &Query<(&Chunk, &ChunkCoordComp)>) -> BlockType {
    if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
        return BlockType::Air;
    }
    let cx = pos.x.div_euclid(CHUNK_SIZE as i32);
    let cz = pos.z.div_euclid(CHUNK_SIZE as i32);
    let lx = pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let ly = pos.y as usize;
    let lz = pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    for (chunk, coord_comp) in chunks.iter() {
        if coord_comp.0.x == cx && coord_comp.0.z == cz {
            return chunk.get(lx, ly, lz);
        }
    }
    BlockType::Air
}
