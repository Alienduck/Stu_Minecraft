use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use crate::{
    net::{EvChatMessage, NetSender},
    player::Player,
};

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (input_handler, key_listener, on_recieve));
    }
}

#[derive(Component)]
pub struct ChatContainer;

#[derive(Component)]
pub struct ChatMessagesArea;

#[derive(Component)]
pub struct ChatInputArea;

#[derive(Component, Default)]
pub struct ChatTypingContent(String);

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(40.0),
                height: Val::Percent(40.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(2.0),
                bottom: Val::Percent(5.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            ChatContainer,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexEnd,
                        padding: UiRect::all(Val::Px(8.0)),
                        margin: UiRect::bottom(Val::Px(5.0)),
                        overflow: Overflow {
                            x: OverflowAxis::Scroll,
                            y: OverflowAxis::Hidden,
                        },
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                    ChatMessagesArea,
                    Visibility::Inherited,
                ))
                .with_children(|messages| {
                    messages.spawn((
                        Text::new("<Steve> Salut ! Il y a un among us par ici ?"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Visibility::Inherited,
                    ));
                    messages.spawn((
                        Text::new("<Alex> Tunic >>> all"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Visibility::Inherited,
                    ));
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(0.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                    ChatInputArea,
                    Visibility::Inherited,
                ))
                .with_children(|input_area| {
                    input_area.spawn((
                        Text::new("> _"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        ChatTypingContent("".into()),
                        TextColor(Color::WHITE),
                        Visibility::Inherited,
                    ));
                });
        });
}

pub fn toggle_chat(mut chat_q: Query<&mut Visibility, With<ChatContainer>>) {
    let Ok(mut chat_container) = chat_q.single_mut() else {
        return;
    };
    if *chat_container == Visibility::Hidden {
        *chat_container = Visibility::Visible;
    } else {
        *chat_container = Visibility::Hidden;
    }
}

fn input_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&mut Player>,
    chat_q: Query<&mut Visibility, With<ChatContainer>>,
) {
    let Ok(mut player) = player_q.single_mut() else {
        return;
    };
    let chat_input = keys.just_pressed(KeyCode::Slash);
    if chat_input {
        player.gpe = !player.gpe;
        toggle_chat(chat_q);
    }
}

fn key_listener(
    mut msg_kb: MessageReader<KeyboardInput>,
    mut chat_content_q: Query<(&mut Text, &mut ChatTypingContent)>,
    mut chat_container_q: Query<&mut Visibility, With<ChatContainer>>,
    mut player_q: Query<&mut Player>, // Pour débloquer le joueur
    sender: Res<NetSender>,
) {
    let Ok(mut visibility) = chat_container_q.single_mut() else {
        return;
    };
    if *visibility == Visibility::Hidden {
        return;
    }

    let Ok((mut text, mut buffer)) = chat_content_q.single_mut() else {
        return;
    };
    let Ok(mut player) = player_q.single_mut() else {
        return;
    };

    for event in msg_kb.read() {
        if event.state == bevy::input::ButtonState::Released {
            continue;
        }

        match &event.logical_key {
            Key::Enter => {
                if !buffer.0.trim().is_empty() {
                    let _ = sender.0.lock().unwrap().send(
                        shared::protocol::ClientPacket::ChatMessage {
                            text: buffer.0.clone(),
                        },
                    );
                }
                buffer.0.clear();
                *visibility = Visibility::Hidden;
                player.gpe = false;
            }
            Key::Escape => {
                buffer.0.clear();
                *visibility = Visibility::Hidden;
                player.gpe = false;
            }
            Key::Space => {
                buffer.0.push(' ');
            }
            Key::Backspace => {
                buffer.0.pop();
            }
            n @ Key::Character(input) => {
                if input == "/" && buffer.0.is_empty() {
                    continue;
                }
                if input.chars().any(|s| s.is_control()) {
                    continue;
                }
                buffer.0.push_str(input);
            }
            _ => {
                continue;
            }
        }

        text.0 = format!("> {}", buffer.0);
    }
}

fn on_recieve(
    mut events: MessageReader<EvChatMessage>,
    mut message_area_q: Query<Entity, With<ChatMessagesArea>>,
    mut commands: Commands,
) {
    let Ok(message_area) = message_area_q.single_mut() else {
        return;
    };
    for event in events.read() {
        commands.entity(message_area).with_children(|parent| {
            parent.spawn((
                Text::new(&event.message),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Visibility::Inherited,
            ));
        });
    }
}
