use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    config::{ConfigPlugin, tile::TileInfoList},
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
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter: "wgpu=error,eternal=trace".to_string(),
                    ..Default::default()
                }),
        )
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
        .add_plugins((ConfigPlugin, WorldPlugin, PlayerPlugin, DebugPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, print_tile_info_list)
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
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.insert_resource(List(asset_server.load("config/tiles.ron")));
}

#[derive(Resource)]
struct List(Handle<TileInfoList>);

fn print_tile_info_list(
    handle: Res<List>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<TileInfoList>>,
) {
    if asset_server.is_loaded(handle.0.id())
        && let Some(tile_info_list) = assets.get(handle.0.id())
    {
        debug!("Tile Info List: {tile_info_list:?}");
    }
}
