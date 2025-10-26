use bevy::{platform::collections::HashMap, prelude::*};
use eternal_config::{
    server::{ConfigAssetUpdated, ConfigServer, Configs},
    tile::{TileConfig, TileConfigList},
};

use crate::tile::{self, TileId, TileInfo};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileRegistry>()
            .add_systems(Startup, setup);
    }
}

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

#[derive(Debug, Default, Clone, Reflect, Deref, DerefMut, Resource)]
pub struct TileRegistry(HashMap<TileId, TileInfo>);

impl TileRegistry {
    pub fn new(map: HashMap<TileId, TileInfo>) -> Self {
        Self(map)
    }

    pub fn get_by_name(&self, name: &str) -> &TileInfo {
        self.0
            .values()
            .find(|info| info.name == name)
            .unwrap_or(&tile::NONE_INFO)
    }

    pub fn get_id_by_name(&self, name: &str) -> TileId {
        self.iter()
            .find_map(|(id, info)| if info.name == name { Some(*id) } else { None })
            .unwrap_or(TileId::none())
    }
}

fn setup(mut config_server: ConfigServer) {
    config_server
        .load::<TileConfigList>("config/tiles.ron")
        .observe(on_tile_config_list_updated);
}

fn on_tile_config_list_updated(
    updated: On<ConfigAssetUpdated>,
    configs: Configs<TileConfigList>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(tile_config_list) = configs.get(updated.id()) else {
        error!("Failed to get tile config list");
        return;
    };

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
                outline,
                blend_tech,
            } = config;

            let info = TileInfo {
                name: name.clone().into(),
                kind: (*kind).into(),
                atlas: asset_server.load(atlas),
                atlas_index: *atlas_index,
                map_color: map_color.into(),
                outline: *outline,
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
