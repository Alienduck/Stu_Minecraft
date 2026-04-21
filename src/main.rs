mod input;
mod player;
mod rendering;
mod world;

use bevy::{prelude::*, window::WindowResolution};
use input::InputPlugin;
use player::PlayerPlugin;
use rendering::RenderingPlugin;
use world::WorldPlugin;

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
        .add_plugins((WorldPlugin, PlayerPlugin, InputPlugin, RenderingPlugin))
        .run();
}
