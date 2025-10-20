use bevy::{platform::collections::HashMap, prelude::*};
use eternal_config::tile::{TileConfig, TileConfigList};

use crate::tile::{self, TileId, TileInfo, TileRegistry};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileRegistry>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                process_tile_info_list.run_if(on_message::<AssetEvent<TileConfigList>>),
            );
    }
}

#[derive(Resource)]
#[expect(unused, reason = "The handle needs to be hold somewhere")]
struct TileInfoHandle(Handle<TileConfigList>);

impl From<eternal_config::tile::TileKind> for crate::tile::TileKind {
    fn from(value: eternal_config::tile::TileKind) -> Self {
        match value {
            eternal_config::tile::TileKind::Terrain => Self::Terrain,
            eternal_config::tile::TileKind::Wall => Self::Wall,
        }
    }
}

impl From<eternal_config::tile::BlendTech> for crate::tile::BlendTech {
    fn from(value: eternal_config::tile::BlendTech) -> Self {
        match value {
            eternal_config::tile::BlendTech::None => Self::None,
            eternal_config::tile::BlendTech::Weight(w) => Self::Weight(w),
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TileInfoHandle(asset_server.load("config/tiles.ron")));
}

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
                        kind: (*kind).into(),
                        atlas: asset_server.load(atlas),
                        atlas_index: *atlas_index,
                        map_color: map_color.into(),
                        blend_tech: blend_tech.unwrap_or_default().into(),
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
