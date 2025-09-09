use bevy::prelude::*;

use crate::tilemap::{Tilemap, TilemapPlugin, TilemapPos, TilemapType};

mod tilemap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(Camera2d);

    let texture = asset_server.load("sheets/terrain.png");
    commands.spawn((
        Tilemap {
            tile_dims: UVec2::new(2, 2),
            texture,
            ..Default::default()
        },
        Name::new("Tilemap"),
        children![
            (Name::new("Tile 0 0"), TilemapPos(0, 0), TilemapType(3),),
            (Name::new("Tile 0 1"), TilemapPos(0, 1), TilemapType(2),),
            (Name::new("Tile 1 0"), TilemapPos(20, 0), TilemapType(1),),
            (Name::new("Tile 1 1"), TilemapPos(20, 1), TilemapType(0),),
        ],
    ));
}
