pub mod camera;
pub mod controller;
pub mod inventory;

use bevy::prelude::*;
use camera::PlayerCameraPlugin;
use controller::PlayerControllerPlugin;
pub use inventory::Inventory;

use crate::net::{EvPlayerJoined, EvPlayerLeft, EvPlayerMoved, EvWelcome};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct RemotePlayer {
    pub id: u64,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerCameraPlugin, PlayerControllerPlugin))
            .add_systems(
                Update,
                (
                    on_welcome_spawn_player,
                    on_player_joined,
                    on_player_left,
                    on_player_moved,
                ),
            );
    }
}

/// Deterministic but varied color from a player id.
/// We spread the hue across the HSL wheel and keep saturation/lightness fixed
/// so every color is vivid and readable.
fn player_color_from_id(id: u64) -> Color {
    // Use a simple hash to spread ids across the hue wheel
    let hash = id.wrapping_mul(2654435761).wrapping_add(id >> 16);
    let hue = (hash % 360) as f32;
    Color::hsl(hue, 0.85, 0.55)
}

/// When player is connected, spawn player
fn on_welcome_spawn_player(
    mut commands: Commands,
    mut ev: MessageReader<EvWelcome>,
    existing: Query<Entity, With<Player>>,
) {
    let Some(welcome) = ev.read().last() else {
        return;
    };

    for e in existing.iter() {
        commands.entity(e).despawn();
    }

    commands.spawn((
        Player,
        Inventory::new(),
        Transform::from_translation(welcome.spawn),
        Visibility::default(),
        controller::Velocity::default(),
        controller::Grounded(false),
        controller::CoyoteTime::default(),
        controller::Sneaking(false),
    ));
}

/// Spawn a remote player using the Among Us GLB scene, tinted with a unique color.
fn on_player_joined(
    mut commands: Commands,
    mut ev: MessageReader<EvPlayerJoined>,
    asset_server: Res<AssetServer>,
    mut _mats: ResMut<Assets<StandardMaterial>>,
) {
    for joined in ev.read() {
        println!(
            "[client] remote player {} joined (id={})",
            joined.name, joined.player_id
        );

        let color = player_color_from_id(joined.player_id);

        // Load the GLB as a scene. The scene is the first (index 0) defined in the file.
        // TODO: change the color where only there are red
        let scene_handle: Handle<Scene> = asset_server.load("models/Sussy_Imposter.glb#Scene0");

        commands.spawn((
            SceneRoot(scene_handle),
            Transform {
                translation: joined.pos,
                scale: Vec3::splat(3.0),
                ..default()
            },
            RemotePlayer {
                id: joined.player_id,
            },
            PlayerTint(color),
        ));
    }
}

/// Marker that carries the desired tint until the scene materials are ready.
#[derive(Component)]
pub struct PlayerTint(Color);

/// Once a scene's children are actually spawned, find every MeshMaterial3d and
/// swap it for a new material with the desired base_color.
/// TODO: change where only vertices are red
pub fn apply_player_tints(
    mut commands: Commands,
    tinted: Query<(Entity, &PlayerTint, &Children), With<RemotePlayer>>,
    children_q: Query<&Children>,
    material_q: Query<Entity, With<MeshMaterial3d<StandardMaterial>>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    for (root, tint, children) in tinted.iter() {
        // Recursively walk children looking for mesh entities
        let mut stack: Vec<Entity> = children.iter().collect();
        let mut found = false;
        while let Some(e) = stack.pop() {
            if material_q.get(e).is_ok() {
                let mat_handle = mats.add(StandardMaterial {
                    base_color: tint.0,
                    perceptual_roughness: 0.6,
                    metallic: 0.1,
                    ..default()
                });
                commands.entity(e).insert(MeshMaterial3d(mat_handle));
                found = true;
            }
            if let Ok(grandchildren) = children_q.get(e) {
                stack.extend(grandchildren.iter());
            }
        }
        if found {
            commands.entity(root).remove::<PlayerTint>();
        }
    }
}

/// Destroy Player mesh
fn on_player_left(
    mut commands: Commands,
    mut ev: MessageReader<EvPlayerLeft>,
    remotes: Query<(Entity, &RemotePlayer)>,
) {
    for left in ev.read() {
        for (entity, rp) in remotes.iter() {
            if rp.id == left.player_id {
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}

fn on_player_moved(
    mut ev: MessageReader<EvPlayerMoved>,
    mut remotes: Query<(&RemotePlayer, &mut Transform)>,
) {
    for moved in ev.read() {
        for (rp, mut transform) in remotes.iter_mut() {
            if rp.id == moved.player_id {
                transform.translation = moved.pos;
                transform.rotation = Quat::from_euler(EulerRot::YXZ, moved.yaw, moved.pitch, 0.0);
                break;
            }
        }
    }
}
