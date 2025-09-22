use bevy::{asset::AssetLoader, prelude::*};

use serde::Deserialize;

use crate::{
    config::{ConfigAssetLoaderError, color::HexColor},
    world::tile::TileKind,
};

pub struct ConfigTilePlugin;

impl Plugin for ConfigTilePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TileConfigList>()
            .init_asset_loader::<TileConfigListAssetLoader>();
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TileConfig {
    pub name: String,
    pub kind: TileKind,
    pub atlas: String,
    pub atlas_index: u16,
    pub map_color: HexColor,
}

#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct TileConfigList(pub Vec<TileConfig>);

#[derive(Default)]
struct TileConfigListAssetLoader;

impl AssetLoader for TileConfigListAssetLoader {
    type Asset = TileConfigList;

    type Settings = ();

    type Error = ConfigAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<TileConfigList, Self::Error> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;
        let tile_list = ron::de::from_bytes(&buffer)?;
        Ok(tile_list)
    }
}
