use bevy::prelude::*;
use std::f32::consts::PI;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_sun, spawn_crosshair))
            .add_systems(Update, update_hotbar_ui);
    }
}

fn spawn_sun(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI / 4.0, -PI / 3.0)),
    ));
}

#[derive(Component)]
struct CrosshairH;

#[derive(Component)]
struct CrosshairV;

#[derive(Component)]
struct HotbarSlot(usize);

fn spawn_crosshair(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                CrosshairH,
            ));
            parent.spawn((
                Node {
                    width: Val::Px(2.0),
                    height: Val::Px(20.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                CrosshairV,
            ));
        });

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            padding: UiRect::bottom(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|row| {
                    for i in 0..8 {
                        row.spawn((
                            Node {
                                width: Val::Px(44.0),
                                height: Val::Px(44.0),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                            BorderColor {
                                top: Color::srgba(0.6, 0.6, 0.6, 0.9),
                                left: Color::srgba(0.6, 0.6, 0.6, 0.9),
                                bottom: Color::srgba(0.6, 0.6, 0.6, 0.9),
                                right: Color::srgba(0.6, 0.6, 0.6, 0.9),
                            },
                            HotbarSlot(i),
                        ));
                    }
                });
        });
}

fn update_hotbar_ui(
    player: Query<&crate::player::inventory::Inventory, With<crate::player::Player>>,
    mut slots: Query<(&HotbarSlot, &mut BorderColor)>,
) {
    let Ok(inv) = player.single() else {
        return;
    };
    for (slot, mut border) in slots.iter_mut() {
        *border = if slot.0 == inv.selected_slot {
            BorderColor {
                top: Color::WHITE,
                right: Color::WHITE,
                bottom: Color::WHITE,
                left: Color::WHITE,
            }
        } else {
            BorderColor {
                top: Color::srgba(0.6, 0.6, 0.6, 0.9),
                left: Color::srgba(0.6, 0.6, 0.6, 0.9),
                bottom: Color::srgba(0.6, 0.6, 0.6, 0.9),
                right: Color::srgba(0.6, 0.6, 0.6, 0.9),
            }
        };
    }
}
