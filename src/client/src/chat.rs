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
pub struct ChatTypingContent {
    pub chars: Vec<char>,
    pub cursor: usize,
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(40.0),
                height: Val::Percent(40.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(2.0),
                bottom: Val::Percent(10.0),
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
                            x: OverflowAxis::Clip,
                            y: OverflowAxis::Clip,
                        },
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                    ChatMessagesArea,
                    Visibility::Inherited,
                ))
                .with_children(|messages| {
                    messages.spawn((
                        Text::new("<Système> Appuyez sur '/' pour discuter."),
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
                        overflow: Overflow {
                            x: OverflowAxis::Clip,
                            y: OverflowAxis::Clip,
                        },
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                    ChatInputArea,
                    Visibility::Inherited,
                ))
                .with_children(|input_area| {
                    input_area.spawn((
                        Text::new("> |"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        ChatTypingContent::default(),
                        TextColor(Color::WHITE),
                        Visibility::Inherited,
                    ));
                });
        });
}

fn input_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&mut Player>,
    mut chat_q: Query<&mut Visibility, With<ChatContainer>>,
) {
    let Ok(mut player) = player_q.single_mut() else {
        return;
    };
    let Ok(mut visibility) = chat_q.single_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::Slash) && *visibility == Visibility::Hidden {
        player.gpe = true;
        *visibility = Visibility::Visible;
    }
}

fn key_listener(
    mut msg_kb: MessageReader<KeyboardInput>,
    mut chat_content_q: Query<(&mut Text, &mut ChatTypingContent)>,
    mut chat_container_q: Query<&mut Visibility, With<ChatContainer>>,
    mut player_q: Query<&mut Player>,
    sender: Res<NetSender>,
) {
    let Ok(mut visibility) = chat_container_q.single_mut() else {
        return;
    };
    if *visibility == Visibility::Hidden {
        return;
    }

    let Ok((mut text, mut content)) = chat_content_q.single_mut() else {
        return;
    };
    let Ok(mut player) = player_q.single_mut() else {
        return;
    };

    for event in msg_kb.read() {
        if event.state == bevy::input::ButtonState::Released {
            continue;
        }

        content.cursor = content.cursor.min(content.chars.len());

        match &event.logical_key {
            Key::Enter => {
                let message: String = content.chars.iter().collect();
                if !message.trim().is_empty() {
                    let _ = sender
                        .0
                        .lock()
                        .unwrap()
                        .send(shared::protocol::ClientPacket::ChatMessage { text: message });
                }
                content.chars.clear();
                content.cursor = 0;
            }
            Key::Escape => {
                content.chars.clear();
                content.cursor = 0;
                *visibility = Visibility::Hidden;
                player.gpe = false;
            }
            Key::ArrowLeft => {
                if content.cursor > 0 {
                    content.cursor -= 1;
                }
            }
            Key::ArrowRight => {
                if content.cursor < content.chars.len() {
                    content.cursor += 1;
                }
            }
            Key::Backspace => {
                if content.cursor > 0 {
                    content.cursor -= 1;
                    if content.cursor < content.chars.len() {
                        let cursor = content.cursor;
                        content.chars.remove(cursor);
                    }
                }
            }
            Key::Delete => {
                if content.cursor < content.chars.len() {
                    let cursor = content.cursor;
                    content.chars.remove(cursor);
                }
            }
            Key::Space => {
                let cursor = content.cursor;
                content.chars.insert(cursor, ' ');
                content.cursor += 1;
            }
            Key::Character(input) => {
                for c in input.chars() {
                    if !c.is_control() {
                        let cursor = content.cursor;
                        content.chars.insert(cursor, c);
                        content.cursor += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let max_len = 45;
    let mut display_start = 0;

    if content.cursor >= max_len {
        display_start = content.cursor - max_len + 5;
    }
    let display_end = (display_start + max_len).min(content.chars.len());

    let mut display_str = String::new();
    if display_start > 0 {
        display_str.push_str("...");
    }

    for i in display_start..display_end {
        if i == content.cursor {
            display_str.push('|');
        }
        display_str.push(content.chars[i]);
    }

    if content.cursor == display_end {
        display_str.push('|');
    }

    if display_end < content.chars.len() {
        display_str.push_str("...");
    }

    text.0 = format!("> {}", display_str);
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
