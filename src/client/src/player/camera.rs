use bevy::prelude::*;

use super::Player;
use crate::input::MovementInput;

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, sync_camera_to_player);
    }
}

#[derive(Component)]
pub struct PlayerCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Transform::default(), PlayerCamera));
}

fn sync_camera_to_player(
    input: Res<MovementInput>,
    player: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Ok(pt) = player.single() else { return };
    let Ok(mut ct) = camera.single_mut() else {
        return;
    };

    ct.translation = pt.translation + Vec3::new(0.0, 1.65, 0.0);
    ct.rotation = Quat::from_euler(EulerRot::YXZ, input.yaw, input.pitch, 0.0);
}
