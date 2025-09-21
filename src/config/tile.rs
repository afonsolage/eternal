use bevy::{asset::AssetLoader, prelude::*};

use serde::Deserialize;

use crate::config::{ConfigAssetLoaderError, color::ConfigColor};

pub struct ConfigTilePlugin;

impl Plugin for ConfigTilePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TileInfoList>()
            .init_asset_loader::<TileInfoListAssetLoader>();
    }
}

#[derive(Debug, Deserialize)]
pub enum TileType {
    Terrain,
}

#[derive(Debug, Deserialize)]
pub struct TileInfo {
    ty: TileType,
    name: String,
    texture: String,
    map_color: ConfigColor,
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct TileInfoList(pub Vec<TileInfo>);

#[derive(Default)]
struct TileInfoListAssetLoader;

impl AssetLoader for TileInfoListAssetLoader {
    type Asset = TileInfoList;

    type Settings = ();

    type Error = ConfigAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<TileInfoList, Self::Error> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;
        let tile_list = ron::de::from_bytes(&buffer)?;
        Ok(tile_list)
    }
}
