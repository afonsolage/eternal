use bevy::{asset::AssetLoader, prelude::*};

use serde::Deserialize;

use crate::{ConfigAssetLoaderError, color::HexColor};

pub struct ConfigTilePlugin;

impl Plugin for ConfigTilePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TileConfigList>()
            .init_asset_loader::<TileConfigListAssetLoader>();
    }
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub enum TileKind {
    #[default]
    Terrain,
    Wall,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub enum BlendTech {
    #[default]
    None,
    Weight(u16),
}

#[derive(Debug, Deserialize, Clone)]
pub struct TileConfig {
    pub name: String,
    pub kind: TileKind,
    pub atlas: String,
    pub atlas_index: u16,
    pub map_color: HexColor,
    pub blend_tech: Option<BlendTech>,
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

        use ron::extensions::Extensions;
        let opts = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);

        let tile_list = opts.from_bytes(&buffer)?;

        Ok(tile_list)
    }
}
