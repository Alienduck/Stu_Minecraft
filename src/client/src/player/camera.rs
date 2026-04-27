// src/client/src/player/camera.rs

use super::Player;
use crate::input::ActionsInput;
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter},
    prelude::*,
    render::view::Hdr,
};

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        // SSAO requires Msaa::Off — attach it to the camera entity.
        // Even without SSAO, Msaa::Off improves performance with HDR bloom.
        app.add_systems(PreStartup, spawn_camera)
            .add_systems(Update, sync_camera_to_player);
    }
}

#[derive(Component)]
pub struct PlayerCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        Hdr,
        Bloom {
            intensity: 0.18,
            low_frequency_boost: 0.4,
            low_frequency_boost_curvature: 0.65,
            high_pass_frequency: 1.0,
            prefilter: BloomPrefilter {
                threshold: 0.85,
                threshold_softness: 0.3,
            },
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Transform::default(),
        PlayerCamera,
        IsDefaultUiCamera,
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
