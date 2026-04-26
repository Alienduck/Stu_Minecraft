use super::Player;
use crate::input::ActionsInput;
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    light::{CascadeShadowConfig, CascadeShadowConfigBuilder},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_camera)
            .add_systems(Startup, spawn_enhanced_sun)
            .add_systems(Update, sync_camera_to_player);
    }
}

#[derive(Component)]
pub struct PlayerCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        Transform::default(),
        PlayerCamera,
        IsDefaultUiCamera,
    ));
}

fn spawn_enhanced_sun(mut commands: Commands) {
    let shadows: CascadeShadowConfig = CascadeShadowConfigBuilder {
        num_cascades: 4,
        maximum_distance: 100.0,
        ..default()
    }
    .into();
    commands.spawn((
        DirectionalLight {
            illuminance: 50_000.0,
            shadows_enabled: true,
            ..default()
        },
        shadows,
    ));
}

fn sync_camera_to_player(
    input: Res<ActionsInput>,
    player_q: Query<(&Transform, &Player), Without<PlayerCamera>>,
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Ok((pt, player)) = player_q.single() else {
        return;
    };
    if player.gpe {
        return;
    }
    let Ok(mut ct) = camera.single_mut() else {
        return;
    };

    ct.translation = pt.translation + Vec3::new(0.0, 1.65, 0.0);
    ct.rotation = Quat::from_euler(EulerRot::YXZ, input.yaw, input.pitch, 0.0);
}
