pub mod camera;
pub mod controller;
pub mod inventory;

use bevy::prelude::*;
use camera::PlayerCameraPlugin;
use controller::PlayerControllerPlugin;
use inventory::Inventory;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerCameraPlugin, PlayerControllerPlugin))
            .add_systems(Startup, spawn_player);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerBody;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        PlayerBody,
        Inventory::new(),
        Transform::from_xyz(0.0, 80.0, 0.0),
        Visibility::default(),
        controller::Velocity::default(),
        controller::Grounded(false),
    ));
}
