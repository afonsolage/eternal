use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    config::ConfigPlugin,
    debug::DebugPlugin,
    player::{Player, PlayerController, PlayerPlugin},
    world::WorldPlugin,
};

mod config;
mod debug;
mod noise;
mod player;
mod ui;
pub mod world;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(LogPlugin {
                level: bevy::log::Level::ERROR,
                filter: "wgpu=error,bevy=warn,eternal=trace".to_string(),
                ..Default::default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
        .add_plugins((ConfigPlugin, WorldPlugin, PlayerPlugin, DebugPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Player,
        PlayerController::default(),
        Sprite {
            image: asset_server.load("sheets/player.png"),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.1),
    ));
}
