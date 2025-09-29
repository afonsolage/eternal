use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    config::tile::{TileConfig, TileConfigList},
    world::{
        genesis::generate_tile_ids,
        renderer::{MapRendererPlugin, tilemap::Tilemap},
        tile::{TileId, TileInfo, TileRegistry},
    },
};

pub mod genesis;
pub mod grid;
pub mod renderer;
pub mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MapRendererPlugin)
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, process_tile_info_list);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tile_ids = generate_tile_ids();
    let tilemap = Tilemap {
        atlas_texture: asset_server.load("sheets/terrain.png"),
        atlas_dims: UVec2::new(4, 4),
        tile_size: Vec2::new(32.0, 32.0),
    };

    commands.insert_resource(TileInfoHandle(asset_server.load("config/tiles.ron")));
    commands.spawn((Name::new("Map"), tilemap, tile_ids));
}

#[derive(Resource)]
#[allow(unused)]
struct TileInfoHandle(Handle<TileConfigList>);

fn process_tile_info_list(
    mut msg_reader: MessageReader<AssetEvent<TileConfigList>>,
    assets: Res<Assets<TileConfigList>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for msg in msg_reader.read() {
        debug!("Event: {msg:?}");
        if let &AssetEvent::Added { id } | &AssetEvent::Modified { id } = msg
            && let Some(tile_config_list) = assets.get(id)
        {
            let map = tile_config_list
                .0
                .iter()
                .enumerate()
                .map(|(idx, config)| {
                    let TileConfig {
                        name,
                        kind,
                        atlas,
                        atlas_index,
                        map_color,
                        blend_tech,
                    } = config;

                    let info = TileInfo {
                        name: name.clone().into(),
                        kind: *kind,
                        atlas: asset_server.load(atlas),
                        atlas_index: *atlas_index,
                        map_color: map_color.0,
                        blend_tech: blend_tech.unwrap_or_default(),
                    };

                    let id = TileId::new(idx as u16);
                    (id, info)
                })
                .chain(std::iter::once((TileId::new(u16::MAX), tile::NONE_INFO)))
                .collect::<HashMap<_, _>>();

            debug!("Loaded tile info list: {map:?}");

            commands.insert_resource(TileRegistry::new(map));
        }
    }
}
