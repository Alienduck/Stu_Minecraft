use bevy::prelude::*;
use std::f32::consts::PI;

use crate::{
    input::BreakState,
    player::{Player, inventory::Inventory},
};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.5, 0.7, 1.0)))
            .init_resource::<ServerTime>()
            .add_systems(Startup, (spawn_lights, spawn_hud))
            .add_systems(
                Update,
                (
                    update_hotbar_ui,
                    update_break_progress,
                    sync_time_from_net,
                    update_day_night_cycle,
                )
                    .chain(),
            );
    }
}

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

#[derive(Component)]
struct Sun;

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 500.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun,
    ));

    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 150.0,
        ..default()
    });
}

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
            // Horizontal bar
            p.spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
            // Vertical bar
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
            // Bar grows from centre outward: offset by -60px so it looks centred
            // TODO: use scaling instead of px
            node.width = Val::Px(pct * 120.0);
            *color = BackgroundColor(Color::srgba(1.0, 0.3 + pct * 0.4, 0.1, 0.9));
        }
    }
}

fn update_day_night_cycle(
    time: Res<Time>,
    mut st: ResMut<ServerTime>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut ambient_query: Query<&mut AmbientLight>,
    mut clear_color: ResMut<ClearColor>,
) {
    let Ok((mut transform, mut light)) = sun_query.single_mut() else {
        return;
    };

    st.local_timer += time.delta_secs();
    let current_time = st.base_time + st.local_timer;

    let cycle_duration = 60.0;
    let t = (current_time % cycle_duration) / cycle_duration;
    let angle = t * PI * 2.0;
    let sun_dist = 500.0;

    transform.translation = Vec3::new(angle.cos() * sun_dist, angle.sin() * sun_dist, 50.0);
    transform.look_at(Vec3::ZERO, Vec3::Y);

    let sun_height = angle.sin();
    light.illuminance = (sun_height * 20_000.0).max(0.0);

    let day_factor = (sun_height * 2.0).clamp(0.0, 1.0);

    let day_sky = Vec3::new(0.5, 0.7, 1.0);
    let night_sky = Vec3::new(0.02, 0.02, 0.05);
    let sky_rgb = night_sky.lerp(day_sky, day_factor);

    *clear_color = ClearColor(Color::srgb(sky_rgb.x, sky_rgb.y, sky_rgb.z));

    if let Ok(mut ambient_light) = ambient_query.single_mut() {
        ambient_light.brightness = 20.0 + (day_factor * 130.0);
    }
}
