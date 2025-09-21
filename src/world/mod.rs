use bevy::prelude::*;

use crate::{
    config::tile::TileInfoList,
    world::{
        genesis::generate_new_map,
        renderer::{MapRendererPlugin, tilemap::Tilemap},
    },
};

pub mod genesis;
pub mod map;
pub mod renderer;
pub mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MapRendererPlugin)
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, load_tile_info_list);
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

    commands.insert_resource(TileInfoHandle(asset_server.load("config/tiles.ron")));
    commands.spawn((Name::new("Map"), tilemap));
}

#[derive(Resource)]
#[allow(unused)]
struct TileInfoHandle(Handle<TileInfoList>);

fn load_tile_info_list(
    mut msg_reader: MessageReader<AssetEvent<TileInfoList>>,
    assets: Res<Assets<TileInfoList>>,
    mut commands: Commands,
) {
    for msg in msg_reader.read() {
        debug!("Event: {msg:?}");
        if let &AssetEvent::Added { id } | &AssetEvent::Modified { id } = msg
            && let Some(tile_info_list) = assets.get(id)
        {
            debug!("Loaded tile info list: {tile_info_list:?}");
            commands.insert_resource(tile_info_list.clone());
        }
    }
}
