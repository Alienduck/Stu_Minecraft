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
            .insert_resource(BreakState::default())
            .add_systems(Startup, grab_cursor)
            .add_systems(Update, handle_keyboard)
            .add_systems(Update, handle_mouse_look)
            .add_systems(Update, handle_scroll)
            .add_systems(Update, handle_breaking)
            .add_systems(Update, handle_place)
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

#[derive(Resource, Default)]
pub struct BreakState {
    pub target: Option<IVec3>,
    pub progress: f32,
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

fn handle_breaking(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    camera: Query<&Transform, With<PlayerCamera>>,
    mut player: Query<&mut Inventory, With<Player>>,
    mut chunks: Query<(&mut Chunk, &ChunkCoord, Entity)>,
    registry: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_mesh_query: Query<&Mesh3d>,
    mut state: ResMut<BreakState>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        state.target = None;
        state.progress = 0.0;
        return;
    }

    let Ok(cam_transform) = camera.single() else {
        return;
    };
    let ray_origin = cam_transform.translation;
    let ray_dir = cam_transform.forward().as_vec3();
    let hit = raycast(ray_origin, ray_dir, 6.0, &chunks);

    if !mouse.pressed(MouseButton::Left) {
        state.target = None;
        state.progress = 0.0;
        return;
    }

    let Some((hit_pos, _)) = hit else {
        state.target = None;
        state.progress = 0.0;
        return;
    };

    let target = IVec3::new(
        hit_pos.x.floor() as i32,
        hit_pos.y.floor() as i32,
        hit_pos.z.floor() as i32,
    );

    if state.target != Some(target) {
        state.target = Some(target);
        state.progress = 0.0;
    }

    let block_type = block_at_world(target, &chunks);
    let break_time = block_type.break_time();
    if break_time <= 0.0 {
        return;
    }

    state.progress += time.delta_secs();

    if state.progress >= break_time {
        state.target = None;
        state.progress = 0.0;
        break_block(
            target,
            block_type,
            &mut player,
            &mut chunks,
            &registry,
            &mut meshes,
            &chunk_mesh_query,
        );
    }
}

fn break_block(
    target: IVec3,
    block_type: BlockType,
    player: &mut Query<&mut Inventory, With<Player>>,
    chunks: &mut Query<(&mut Chunk, &ChunkCoord, Entity)>,
    registry: &BlockRegistry,
    meshes: &mut Assets<Mesh>,
    chunk_mesh_query: &Query<&Mesh3d>,
) {
    let cx = target.x.div_euclid(CHUNK_SIZE as i32);
    let cz = target.z.div_euclid(CHUNK_SIZE as i32);
    let lx = target.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let ly = target.y as usize;
    let lz = target.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    for (mut chunk, coord, entity) in chunks.iter_mut() {
        if coord.x != cx || coord.z != cz {
            continue;
        }

        chunk.set(lx, ly, lz, BlockType::Air);

        if let Ok(mut inv) = player.single_mut() {
            inv.add(block_type);
        }

        let new_mesh = build_chunk_mesh(&chunk, registry);
        if let Ok(mesh3d) = chunk_mesh_query.get(entity) {
            if let Some(mesh) = meshes.get_mut(&mesh3d.0) {
                *mesh = new_mesh;
            }
        }
        break;
    }
}

fn handle_place(
    mouse: Res<ButtonInput<MouseButton>>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    camera: Query<&Transform, With<PlayerCamera>>,
    mut player: Query<&mut Inventory, With<Player>>,
    mut chunks: Query<(&mut Chunk, &ChunkCoord, Entity)>,
    registry: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_mesh_query: Query<&Mesh3d>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(cam_transform) = camera.single() else {
        return;
    };
    let Ok(mut inventory) = player.single_mut() else {
        return;
    };

    let ray_origin = cam_transform.translation;
    let ray_dir = cam_transform.forward().as_vec3();

    let Some((hit_pos, normal)) = raycast(ray_origin, ray_dir, 6.0, &chunks) else {
        return;
    };

    let place_pos = hit_pos + normal;
    let bx = place_pos.x.floor() as i32;
    let by = place_pos.y.floor() as i32;
    let bz = place_pos.z.floor() as i32;

    if by < 0 || by >= CHUNK_HEIGHT as i32 {
        return;
    }

    let block = inventory.selected_block();
    if inventory.count(block) == 0 {
        return;
    }

    let cx = bx.div_euclid(CHUNK_SIZE as i32);
    let cz = bz.div_euclid(CHUNK_SIZE as i32);
    let lx = bx.rem_euclid(CHUNK_SIZE as i32) as usize;
    let ly = by as usize;
    let lz = bz.rem_euclid(CHUNK_SIZE as i32) as usize;

    for (mut chunk, coord, entity) in chunks.iter_mut() {
        if coord.x != cx || coord.z != cz {
            continue;
        }

        chunk.set(lx, ly, lz, block);
        inventory.remove(block);

        let new_mesh = build_chunk_mesh(&chunk, &registry);
        if let Ok(mesh3d) = chunk_mesh_query.get(entity) {
            if let Some(mesh) = meshes.get_mut(&mesh3d.0) {
                *mesh = new_mesh;
            }
        }
        break;
    }
}

fn block_at_world(pos: IVec3, chunks: &Query<(&mut Chunk, &ChunkCoord, Entity)>) -> BlockType {
    if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
        return BlockType::Air;
    }
    let cx = pos.x.div_euclid(CHUNK_SIZE as i32);
    let cz = pos.z.div_euclid(CHUNK_SIZE as i32);
    let lx = pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let ly = pos.y as usize;
    let lz = pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    for (chunk, coord, _) in chunks.iter() {
        if coord.x == cx && coord.z == cz {
            return chunk.get(lx, ly, lz);
        }
    }
    BlockType::Air
}

fn raycast(
    origin: Vec3,
    dir: Vec3,
    max_dist: f32,
    chunks: &Query<(&mut Chunk, &ChunkCoord, Entity)>,
) -> Option<(Vec3, Vec3)> {
    const STEPS: usize = 120;
    let step = max_dist / STEPS as f32;
    let mut last_pos = Vec3::new(origin.x.floor(), origin.y.floor(), origin.z.floor());

    for i in 1..STEPS {
        let pos = origin + dir * (i as f32 * step);
        let bx = pos.x.floor() as i32;
        let by = pos.y.floor() as i32;
        let bz = pos.z.floor() as i32;

        if by < 0 || by >= CHUNK_HEIGHT as i32 {
            continue;
        }

        let cx = bx.div_euclid(CHUNK_SIZE as i32);
        let cz = bz.div_euclid(CHUNK_SIZE as i32);
        let lx = bx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let ly = by as usize;
        let lz = bz.rem_euclid(CHUNK_SIZE as i32) as usize;

        for (chunk, coord, _) in chunks.iter() {
            if coord.x == cx && coord.z == cz {
                if chunk.get(lx, ly, lz).is_solid() {
                    let hit = Vec3::new(bx as f32, by as f32, bz as f32);
                    let normal = (last_pos - hit).normalize().round();
                    return Some((hit, normal));
                }
            }
        }

        last_pos = Vec3::new(bx as f32, by as f32, bz as f32);
    }
    None
}
