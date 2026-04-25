pub mod camera;
pub mod controller;
pub mod inventory;

use bevy::prelude::*;
use camera::PlayerCameraPlugin;
use controller::PlayerControllerPlugin;
pub use inventory::Inventory;

use crate::net::{EvPlayerJoined, EvPlayerLeft, EvPlayerMoved, EvWelcome};

#[derive(Component, Default)]
pub struct Player {
    /// Game Processed Event, is when the player is interacting with UI
    pub gpe: bool,
}

#[derive(Component)]
pub struct RemotePlayer {
    pub id: u64,
}

#[derive(Component)]
pub struct NameTag {
    pub target: Entity,
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
        Player::default(),
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

        // Load the GLB as a scene. The scene is the first (index 0) defined in the file.
        // TODO: change the color where only there are red
        let scene_handle: Handle<Scene> = asset_server.load("models/Sussy_Imposter.glb#Scene0");

        let player_entity = commands
            .spawn((
                SceneRoot(scene_handle),
                Transform {
                    translation: joined.pos,
                    scale: Vec3::splat(3.0),
                    ..default()
                },
                RemotePlayer {
                    id: joined.player_id,
                },
            ))
            .id();

        commands.spawn((
            Text::new(joined.name.clone()),
            TextColor(Color::WHITE),
            BackgroundColor::DEFAULT,
            TextFont {
                font_size: 24.0,
                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                ..default()
            },
            NameTag {
                target: player_entity,
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
