// src/client/src/rendering/mod.rs

pub mod fog_pass;
pub mod lens_flare;
pub mod water_material;

use bevy::prelude::*;
use std::f32::consts::PI;

use crate::{
    input::BreakState,
    player::{NameTag, Player, RemotePlayer, inventory::Inventory},
};

use fog_pass::FogPostProcessPlugin;
use lens_flare::LensFlarePlugin;
use water_material::WaterMaterialPlugin;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WaterMaterialPlugin, LensFlarePlugin, FogPostProcessPlugin))
            .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 1.0)))
            .init_resource::<ServerTime>()
            .add_systems(Startup, (spawn_lights, spawn_hud, spawn_celestial_bodies))
            .add_systems(
                Update,
                (
                    update_hotbar_ui,
                    update_break_progress,
                    sync_time_from_net,
                    update_day_night_cycle,
                    update_billboard,
                )
                    .chain(),
            );
    }
}

// ── Markers ──────────────────────────────────────────────────────────────────

/// Marks the sun directional light AND the sun mesh.
#[derive(Component)]
pub struct Sun;

/// Marks the moon mesh.
#[derive(Component)]
pub struct Moon;

// ── Resources ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct ServerTime {
    base_time: f32,
    local_timer: f32,
}

fn sync_time_from_net(mut ev: MessageReader<crate::net::EvTimeUpdate>, mut st: ResMut<ServerTime>) {
    if let Some(e) = ev.read().last() {
        st.base_time = e.time;
        st.local_timer = 0.0;
    }
}

const ORBIT_RADIUS: f32 = 450.0;

// ── Startup ───────────────────────────────────────────────────────────────────

fn spawn_lights(mut commands: Commands) {
    // ONE directional light for the sun — driven by update_day_night_cycle.
    // Do NOT spawn another one in camera.rs.
    let shadows = bevy::light::CascadeShadowConfigBuilder {
        num_cascades: 4,
        maximum_distance: 120.0,
        first_cascade_far_bound: 6.0,
        overlap_proportion: 0.3,
        ..default()
    }
    .build();

    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 1.8,
            ..default()
        },
        shadows,
        Transform::from_xyz(0.0, 500.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun,
    ));

    commands.spawn(AmbientLight {
        color: Color::srgb(0.6, 0.65, 0.8),
        brightness: 180.0,
        ..default()
    });
}

fn spawn_celestial_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    // Sun mesh — strong emissive so Bloom picks it up
    let sun_mesh = meshes.add(Sphere::new(18.0).mesh().ico(3).unwrap());
    let sun_mat = mats.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.97, 0.5),
        emissive: LinearRgba::new(6.0, 5.0, 1.5, 1.0),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(sun_mesh),
        MeshMaterial3d(sun_mat),
        Transform::from_xyz(ORBIT_RADIUS, 0.0, 50.0),
        Sun,
    ));

    // Moon mesh
    let moon_mesh = meshes.add(Sphere::new(10.0).mesh().ico(2).unwrap());
    let moon_mat = mats.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.87, 0.92),
        emissive: LinearRgba::new(0.2, 0.22, 0.28, 1.0),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(moon_mesh),
        MeshMaterial3d(moon_mat),
        Transform::from_xyz(-ORBIT_RADIUS, 0.0, 50.0),
        Moon,
    ));
}

// ── HUD ───────────────────────────────────────────────────────────────────────

#[derive(Component)]
struct HotbarSlot(usize);

#[derive(Component)]
struct HotbarCount(usize);

#[derive(Component)]
struct BreakProgressBar;

fn spawn_hud(mut commands: Commands) {
    // Crosshair
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
            p.spawn((
                Node {
                    width: Val::Px(2.0),
                    height: Val::Px(16.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
        });

    // Break progress bar
    commands.spawn((
        Node {
            width: Val::Px(0.0),
            height: Val::Px(6.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(80.0),
            left: Val::Percent(50.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.3, 0.1, 0.9)),
        BreakProgressBar,
    ));

    // Hotbar
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            padding: UiRect::bottom(Val::Px(12.0)),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|p| {
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|row| {
                for i in 0..8 {
                    row.spawn((
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::FlexEnd,
                            padding: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.75)),
                        BorderColor {
                            top: Color::srgba(0.5, 0.5, 0.5, 0.8),
                            left: Color::srgba(0.5, 0.5, 0.5, 0.8),
                            bottom: Color::srgba(0.5, 0.5, 0.5, 0.8),
                            right: Color::srgba(0.5, 0.5, 0.5, 0.8),
                        },
                        HotbarSlot(i),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            Text::new("64"),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            HotbarCount(i),
                        ));
                    });
                }
            });
        });
}

// ── Update ────────────────────────────────────────────────────────────────────

fn update_hotbar_ui(
    player: Query<&Inventory, With<Player>>,
    mut slots: Query<(&HotbarSlot, &mut BorderColor)>,
    mut counts: Query<(&HotbarCount, &mut Text)>,
) {
    let Ok(inv) = player.single() else { return };
    for (slot, mut border) in slots.iter_mut() {
        let color = if slot.0 == inv.selected_slot {
            Color::WHITE
        } else {
            Color::srgba(0.5, 0.5, 0.5, 0.8)
        };
        *border = BorderColor {
            top: color,
            left: color,
            bottom: color,
            right: color,
        };
    }
    for (count_marker, mut text) in counts.iter_mut() {
        **text = inv.slot_count(count_marker.0).to_string();
    }
}

fn update_break_progress(
    state: Res<BreakState>,
    mut bar: Query<(&mut Node, &mut BackgroundColor), With<BreakProgressBar>>,
) {
    let Ok((mut node, mut color)) = bar.single_mut() else {
        return;
    };
    match state.target {
        None => {
            node.width = Val::Px(0.0);
        }
        Some(_) => {
            let pct = (state.progress / 2.0).clamp(0.0, 1.0);
            node.width = Val::Px(pct * 120.0);
            *color = BackgroundColor(Color::srgba(1.0, 0.3 + pct * 0.4, 0.1, 0.9));
        }
    }
}

fn update_day_night_cycle(
    time: Res<Time>,
    mut st: ResMut<ServerTime>,
    // Query only the entity that has BOTH Sun AND DirectionalLight
    mut sun_light_q: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    // Query only the Sun entity that has a Mesh3d (not the light)
    mut sun_mesh_q: Query<
        &mut Transform,
        (
            With<Sun>,
            With<Mesh3d>,
            Without<DirectionalLight>,
            Without<Moon>,
        ),
    >,
    mut moon_mesh_q: Query<&mut Transform, (With<Moon>, Without<DirectionalLight>, Without<Sun>)>,
    mut ambient_query: Query<&mut AmbientLight>,
    mut clear_color: ResMut<ClearColor>,
) {
    st.local_timer += time.delta_secs();
    let current_time = st.base_time + st.local_timer;

    let cycle_duration = 600.0_f32;
    let t = (current_time % cycle_duration) / cycle_duration;
    let angle = t * PI * 2.0;

    let sun_pos = Vec3::new(angle.cos() * ORBIT_RADIUS, angle.sin() * ORBIT_RADIUS, 50.0);

    if let Ok((mut transform, mut light)) = sun_light_q.single_mut() {
        transform.translation = sun_pos;
        transform.look_at(Vec3::ZERO, Vec3::Y);
        let sun_height = angle.sin();
        light.illuminance = (sun_height * 22_000.0).max(0.0);
    }

    if let Ok(mut t) = sun_mesh_q.single_mut() {
        t.translation = sun_pos;
    }

    if let Ok(mut t) = moon_mesh_q.single_mut() {
        let moon_angle = angle + PI;
        t.translation = Vec3::new(
            moon_angle.cos() * ORBIT_RADIUS,
            moon_angle.sin() * ORBIT_RADIUS,
            50.0,
        );
    }

    let sun_height = angle.sin();
    let day_factor = (sun_height * 2.0).clamp(0.0, 1.0);

    let day_sky = Vec3::new(0.40, 0.65, 1.00);
    let dusk_sky = Vec3::new(0.72, 0.30, 0.08);
    let night_sky = Vec3::new(0.02, 0.02, 0.06);

    let sky_rgb = if sun_height > 0.0 {
        let dusk_factor = 1.0 - sun_height.abs().min(1.0);
        let daytime = day_sky.lerp(dusk_sky, dusk_factor * dusk_factor);
        daytime * day_factor + night_sky * (1.0 - day_factor)
    } else {
        night_sky
    };

    *clear_color = ClearColor(Color::srgb(sky_rgb.x, sky_rgb.y, sky_rgb.z));

    if let Ok(mut al) = ambient_query.single_mut() {
        al.brightness = 25.0 + day_factor * 155.0;
    }
}

fn update_billboard(
    mut commands: Commands,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    player_q: Query<&GlobalTransform, (With<RemotePlayer>, Without<NameTag>)>,
    mut tags_q: Query<(Entity, &mut Node, &NameTag)>,
) {
    let Some((camera, cam_global)) = camera_q.iter().next() else {
        return;
    };
    for (entity, mut node, tag) in tags_q.iter_mut() {
        if let Ok(player_global) = player_q.get(tag.target) {
            let world_pos = player_global.translation() + Vec3::new(0.0, 2.2, 0.0);
            if let Ok(screen_pos) = camera.world_to_viewport(cam_global, world_pos) {
                node.display = Display::Flex;
                node.left = Val::Px(screen_pos.x - 30.0);
                node.top = Val::Px(screen_pos.y);
            } else {
                node.display = Display::None;
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
}
