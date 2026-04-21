use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    player::{Player, camera::PlayerCamera, inventory::Inventory},
    world::{
        chunk::{CHUNK_HEIGHT, CHUNK_SIZE, Chunk, ChunkCoord, build_chunk_mesh},
        registry::{BlockRegistry, BlockType},
    },
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MovementInput::default())
            .add_systems(Startup, grab_cursor)
            .add_systems(Update, handle_keyboard)
            .add_systems(Update, handle_mouse_look)
            .add_systems(Update, handle_scroll)
            .add_systems(Update, handle_block_interaction)
            .add_systems(Update, toggle_cursor_grab);
    }
}

#[derive(Resource, Default)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprinting: bool,
    pub yaw: f32,
    pub pitch: f32,
}

fn grab_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let locked = cursor.grab_mode == CursorGrabMode::Locked;
        cursor.grab_mode = if locked {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };
        cursor.visible = locked;
    }
}

fn handle_keyboard(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<MovementInput>) {
    input.forward = keys.pressed(KeyCode::KeyW);
    input.backward = keys.pressed(KeyCode::KeyS);
    input.left = keys.pressed(KeyCode::KeyA);
    input.right = keys.pressed(KeyCode::KeyD);
    input.jump = keys.pressed(KeyCode::Space);
    input.sprinting = keys.pressed(KeyCode::ShiftLeft);
}

fn handle_mouse_look(
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    mut motion: MessageReader<MouseMotion>,
    mut input: ResMut<MovementInput>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    const SENSITIVITY: f32 = 0.002;
    for event in motion.read() {
        input.yaw -= event.delta.x * SENSITIVITY;
        input.pitch = (input.pitch - event.delta.y * SENSITIVITY).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
    }
}

fn handle_scroll(
    mut scroll: MessageReader<MouseWheel>,
    mut player: Query<&mut Inventory, With<Player>>,
) {
    let Ok(mut inv) = player.single_mut() else {
        return;
    };
    for event in scroll.read() {
        inv.scroll(event.y);
    }
}

fn handle_block_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    camera: Query<&Transform, With<PlayerCamera>>,
    player: Query<&Inventory, With<Player>>,
    mut chunks: Query<(&mut Chunk, &ChunkCoord, &Transform, Entity)>,
    registry: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_mesh_query: Query<&Mesh3d>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let break_pressed = mouse.just_pressed(MouseButton::Left);
    let place_pressed = mouse.just_pressed(MouseButton::Right);

    if !break_pressed && !place_pressed {
        return;
    }

    let Ok(cam_transform) = camera.single() else {
        return;
    };
    let Ok(inventory) = player.single() else {
        return;
    };

    let ray_origin = cam_transform.translation;
    let ray_dir = cam_transform.forward().as_vec3();

    let Some((hit_world, normal)) = raycast(ray_origin, ray_dir, 6.0, &chunks) else {
        return;
    };

    let target_world = if place_pressed {
        hit_world + normal
    } else {
        hit_world
    };

    let bx = target_world.x.floor() as i32;
    let by = target_world.y.floor() as i32;
    let bz = target_world.z.floor() as i32;

    if by < 0 || by >= CHUNK_HEIGHT as i32 {
        return;
    }

    let cx = bx.div_euclid(CHUNK_SIZE as i32);
    let cz = bz.div_euclid(CHUNK_SIZE as i32);
    let lx = bx.rem_euclid(CHUNK_SIZE as i32) as usize;
    let ly = by as usize;
    let lz = bz.rem_euclid(CHUNK_SIZE as i32) as usize;

    for (mut chunk, coord, _, entity) in chunks.iter_mut() {
        if coord.x != cx || coord.z != cz {
            continue;
        }

        if break_pressed {
            chunk.set(lx, ly, lz, BlockType::Air);
        } else {
            chunk.set(lx, ly, lz, inventory.selected_block());
        }

        let new_mesh = build_chunk_mesh(&chunk, &registry);
        if let Ok(mesh3d) = chunk_mesh_query.get(entity) {
            if let Some(mesh) = meshes.get_mut(&mesh3d.0) {
                *mesh = new_mesh;
            }
        }
        break;
    }
}

fn raycast(
    origin: Vec3,
    dir: Vec3,
    max_dist: f32,
    chunks: &Query<(&mut Chunk, &ChunkCoord, &Transform, Entity)>,
) -> Option<(Vec3, Vec3)> {
    const STEPS: usize = 120;
    let step = max_dist / STEPS as f32;
    let mut last_air = origin;

    for i in 0..STEPS {
        let pos = origin + dir * (i as f32 * step);
        let bx = pos.x.floor() as i32;
        let by = pos.y.floor() as i32;
        let bz = pos.z.floor() as i32;

        if by < 0 {
            continue;
        }

        let cx = bx.div_euclid(CHUNK_SIZE as i32);
        let cz = bz.div_euclid(CHUNK_SIZE as i32);
        let lx = bx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let ly = by as usize;
        let lz = bz.rem_euclid(CHUNK_SIZE as i32) as usize;

        if ly >= CHUNK_HEIGHT {
            continue;
        }

        for (chunk, coord, _, _) in chunks.iter() {
            if coord.x == cx && coord.z == cz {
                if chunk.get(lx, ly, lz).is_solid() {
                    let hit = Vec3::new(bx as f32, by as f32, bz as f32);
                    let normal = (last_air - hit).normalize().round();
                    return Some((hit, normal));
                }
            }
        }

        last_air = Vec3::new(bx as f32, by as f32, bz as f32);
    }
    None
}
