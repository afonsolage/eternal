use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::ui::UiPlugin;
use crate::world::grid;
use crate::{
    config::ConfigPlugin,
    debug::DebugPlugin,
    player::{Player, PlayerPlugin},
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
        .add_plugins((
            ConfigPlugin,
            WorldPlugin,
            PlayerPlugin,
            UiPlugin,
            DebugPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Player,
        Transform::from_translation(grid::grid_to_world(127, 127).extend(0.1)),
    ));
}
