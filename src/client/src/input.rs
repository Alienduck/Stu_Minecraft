use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use shared::chunk::{CHUNK_HEIGHT, CHUNK_SIZE};

use crate::{
    net::NetSender,
    player::{Player, camera::PlayerCamera, controller::block_at_world, inventory::Inventory},
    world::{Chunk, ChunkCoordComp},
};

use shared::protocol::ClientPacket;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MovementInput::default())
            .insert_resource(BreakState::default())
            .add_systems(Startup, grab_cursor)
            .add_systems(
                Update,
                (
                    handle_keyboard,
                    handle_mouse,
                    handle_mouse_look,
                    handle_scroll,
                    handle_breaking,
                    handle_place,
                    toggle_cursor_grab,
                    send_player_position,
                ),
            );
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
    pub sneaking: bool,
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
    input.sneaking = keys.pressed(KeyCode::ControlLeft);
}

fn handle_mouse(mouse: Res<ButtonInput<MouseButton>>, mut input: ResMut<MovementInput>) {
    input.sneaking = mouse.pressed(MouseButton::Back);
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
    for e in motion.read() {
        input.yaw -= e.delta.x * SENSITIVITY;
        input.pitch = (input.pitch - e.delta.y * SENSITIVITY).clamp(
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
    for e in scroll.read() {
        inv.scroll(e.y);
    }
}

fn send_player_position(
    input: Res<MovementInput>,
    player: Query<&Transform, With<Player>>,
    sender: Res<NetSender>,
) {
    let Ok(t) = player.single() else { return };
    let _ = sender.0.lock().unwrap().send(ClientPacket::Move {
        x: t.translation.x,
        y: t.translation.y,
        z: t.translation.z,
        yaw: input.yaw,
        pitch: input.pitch,
    });
}

fn handle_breaking(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    camera: Query<&Transform, With<PlayerCamera>>,
    mut player: Query<&mut Inventory, With<Player>>,
    chunks: Query<(&Chunk, &ChunkCoordComp)>,
    mut state: ResMut<BreakState>,
    sender: Res<NetSender>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        state.target = None;
        state.progress = 0.0;
        return;
    }

    let Ok(cam) = camera.single() else { return };
    let hit = raycast(cam.translation, cam.forward().as_vec3(), 6.0, &chunks);

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
    if break_time <= 0.0 || break_time == f32::MAX {
        return;
    }

    state.progress += time.delta_secs();

    if state.progress >= break_time {
        state.target = None;
        state.progress = 0.0;

        if let Ok(mut inv) = player.single_mut() {
            inv.add(block_type);
        }

        let _ = sender.0.lock().unwrap().send(ClientPacket::BreakBlock {
            wx: target.x,
            wy: target.y,
            wz: target.z,
        });
    }
}

fn handle_place(
    mouse: Res<ButtonInput<MouseButton>>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    camera: Query<&Transform, With<PlayerCamera>>,
    mut player: Query<&mut Inventory, With<Player>>,
    chunks: Query<(&Chunk, &ChunkCoordComp)>,
    sender: Res<NetSender>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(cam) = camera.single() else { return };
    let Ok(mut inv) = player.single_mut() else {
        return;
    };

    let Some((hit_pos, normal)) = raycast(cam.translation, cam.forward().as_vec3(), 6.0, &chunks)
    else {
        return;
    };

    let place = hit_pos + normal;
    let bx = place.x.floor() as i32;
    let by = place.y.floor() as i32;
    let bz = place.z.floor() as i32;

    if by < 0 || by >= CHUNK_HEIGHT as i32 {
        return;
    }

    let block = inv.selected_block();
    if inv.count(block) == 0 {
        return;
    }

    inv.remove(block);

    let _ = sender.0.lock().unwrap().send(ClientPacket::PlaceBlock {
        wx: bx,
        wy: by,
        wz: bz,
        block,
    });
}

pub fn raycast(
    origin: Vec3,
    dir: Vec3,
    max_dist: f32,
    chunks: &Query<(&Chunk, &ChunkCoordComp)>,
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

        for (chunk, coord_comp) in chunks.iter() {
            if coord_comp.0.x == cx && coord_comp.0.z == cz {
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
