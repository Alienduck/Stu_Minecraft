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
    ));
}

/// Create player mesh
fn on_player_joined(
    mut commands: Commands,
    mut ev: MessageReader<EvPlayerJoined>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    for joined in ev.read() {
        println!("[client] remote player {} joined", joined.name);
        // Spawn a simple colored cube as ghost representation
        // TODO: Make Among Us skin (thx TitiLucas77 for Idea)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.6, 1.8, 0.6))),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                ..default()
            })),
            Transform::from_translation(joined.pos),
            RemotePlayer {
                id: joined.player_id,
            },
        ));
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
