use bevy::prelude::*;

use crate::world::{
    genesis::generate_new_map,
    renderer::{MapRendererPlugin, tilemap::Tilemap},
};

pub mod genesis;
pub mod map;
pub mod renderer;
pub mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MapRendererPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map = generate_new_map();
    let tilemap = Tilemap {
        atlas_texture: asset_server.load("sheets/terrain.png"),
        atlas_dims: UVec2::new(4, 4),
        tile_size: Vec2::new(32.0, 32.0),
        map,
    };

    commands.spawn((Name::new("Map"), tilemap));
}
