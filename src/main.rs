use bevy::log::LogPlugin;
use bevy::prelude::*;

use crate::tilemap::{Tilemap, TilemapIndex, TilemapPlugin, TilemapPos};

mod tilemap;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter: "wgpu=error,naga=warn,eternal=trace".to_string(),
                    ..Default::default()
                }),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2d);

    let atlas_texture = asset_server.load("sheets/terrain.png");
    let tilemap_entity = commands
        .spawn((
            Tilemap {
                atlas_texture,
                atlas_dims: UVec2::new(2, 2),
                tile_size: Vec2::new(32.0, 32.0),
                ..Default::default()
            },
            Name::new("Tilemap"),
        ))
        .id();

    for x in -64..64 {
        for y in -64..64 {
            commands.entity(tilemap_entity).with_child((
                Name::new(format!("Tile {x} {y}")),
                TilemapPos(x, y),
                TilemapIndex(0),
            ));
        }
    }
}
