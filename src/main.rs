use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    debug::DebugPlugin,
    noise::Noise,
    player::{Player, PlayerController, PlayerPlugin},
    tilemap::{Tilemap, TilemapIndex, TilemapPlugin, TilemapPos},
};

mod debug;
mod noise;
mod player;
mod tilemap;
mod ui;

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
        .add_plugins((TilemapPlugin, PlayerPlugin, DebugPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
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

    let noise = Noise::new();
    let atlas_texture = asset_server.load("sheets/terrain.png");
    // let atlas_texture = asset_server.load("sheets/tilemap_debug.png");

    commands
        .spawn((
            Tilemap {
                atlas_texture,
                atlas_dims: UVec2::new(4, 4),
                tile_size: Vec2::new(32.0, 32.0),
                ..Default::default()
            },
            Name::new("Tilemap"),
        ))
        .with_children(|parent| {
            for x in 0..16 {
                for y in 0..16 {
                    let h = noise.stone(x as f32, y as f32);
                    let i = if h > 50 { 1 } else { 2 };

                    info!("{i}");

                    parent.spawn((
                        Name::new(format!("Tile {x} {y}")),
                        TilemapPos(x, y),
                        TilemapIndex(i),
                    ));
                }
            }
        });
}
