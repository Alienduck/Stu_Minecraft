use bevy::prelude::*;

use crate::input::MovementInput;

use super::Player;

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
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut cam_transform) = camera.single_mut() else {
        return;
    };

    let eye_offset = Vec3::new(0.0, 1.65, 0.0);
    cam_transform.translation = player_transform.translation + eye_offset;
    cam_transform.rotation = Quat::from_euler(EulerRot::YXZ, input.yaw, input.pitch, 0.0);
}
