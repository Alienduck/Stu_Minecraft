use bevy::prelude::*;

use crate::{
    input::MovementInput,
    world::chunk::{CHUNK_SIZE, Chunk, ChunkCoord},
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

pub fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Grounded), With<Player>>,
) {
    for (mut vel, mut grounded) in query.iter_mut() {
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
    chunks: Query<(&Chunk, &crate::world::chunk::ChunkCoord, &Transform)>,
    mut query: Query<
        (&mut Transform, &mut Velocity, &mut Grounded),
        (With<Player>, Without<Chunk>),
    >,
) {
    let dt = time.delta_secs();

    for (mut transform, mut vel, mut grounded) in query.iter_mut() {
        let speed = if input.sprinting {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        let forward = Vec3::new(input.yaw.sin(), 0.0, input.yaw.cos());
        let right = Vec3::new(forward.z, 0.0, -forward.x);

        let mut wish_dir = Vec3::ZERO;
        if input.forward {
            wish_dir -= forward;
        }
        if input.backward {
            wish_dir += forward;
        }
        if input.left {
            wish_dir -= right;
        }
        if input.right {
            wish_dir += right;
        }

        if wish_dir.length_squared() > 0.0 {
            wish_dir = wish_dir.normalize() * speed;
        }

        vel.0.x = wish_dir.x;
        vel.0.z = wish_dir.z;

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

fn try_move(
    pos: Vec3,
    delta: Vec3,
    chunks: &Query<(&Chunk, &crate::world::chunk::ChunkCoord, &Transform)>,
) -> Vec3 {
    let new_pos = pos + delta;
    if !collides_with_world(new_pos, chunks) {
        new_pos
    } else {
        pos
    }
}

fn collides_with_world(
    pos: Vec3,
    chunks: &Query<(&Chunk, &crate::world::chunk::ChunkCoord, &Transform)>,
) -> bool {
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

    for corner in &corners {
        if is_solid_at(*corner, chunks) {
            return true;
        }
    }
    false
}

fn is_on_ground(
    pos: Vec3,
    chunks: &Query<(&Chunk, &crate::world::chunk::ChunkCoord, &Transform)>,
) -> bool {
    let hw = PLAYER_WIDTH * 0.5;
    let feet = pos.y - 0.05;
    let corners = [
        Vec3::new(pos.x - hw, feet, pos.z - hw),
        Vec3::new(pos.x + hw, feet, pos.z - hw),
        Vec3::new(pos.x - hw, feet, pos.z + hw),
        Vec3::new(pos.x + hw, feet, pos.z + hw),
    ];
    corners.iter().any(|c| is_solid_at(*c, chunks))
}

pub fn is_solid_at(world_pos: Vec3, chunks: &Query<(&Chunk, &ChunkCoord, &Transform)>) -> bool {
    let bx = world_pos.x.floor() as i32;
    let by = world_pos.y.floor() as i32;
    let bz = world_pos.z.floor() as i32;

    if by < 0 {
        return true;
    }

    let cx = bx.div_euclid(CHUNK_SIZE as i32);
    let cz = bz.div_euclid(CHUNK_SIZE as i32);
    let lx = bx.rem_euclid(CHUNK_SIZE as i32) as usize;
    let ly = by as usize;
    let lz = bz.rem_euclid(CHUNK_SIZE as i32) as usize;

    for (chunk, coord, _) in chunks.iter() {
        if coord.x == cx && coord.z == cz {
            if ly < crate::world::chunk::CHUNK_HEIGHT {
                return chunk.get(lx, ly, lz).is_solid();
            }
        }
    }
    false
}
