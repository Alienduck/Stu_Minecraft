mod chat;
mod input;
mod net;
mod player;
mod rendering;
mod world;

use bevy::{prelude::*, window::WindowResolution};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Stu Minecraft".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            net::NetPlugin,
            world::WorldPlugin,
            player::PlayerPlugin,
            input::InputPlugin,
            rendering::RenderingPlugin,
            chat::ChatPlugin,
        ))
        .run();
}
