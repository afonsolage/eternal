use bevy::prelude::*;

use crate::{
    ConfigAssetLoaderError,
    color::HexColor,
    loader::{ConfigAssetLoader, ConfigParser},
};

pub struct ConfigTilePlugin;

impl Plugin for ConfigTilePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TileConfigList>()
            .init_asset_loader::<ConfigAssetLoader<TileConfigList>>();
    }
}

#[derive(Debug, Default, Clone, Copy, Reflect)]
pub enum TileKind {
    #[default]
    Terrain,
    Wall,
}

#[derive(Debug, Default, Clone, Copy, Reflect)]
pub enum BlendTech {
    #[default]
    None,
    Weight(u16),
}

#[derive(Debug, Reflect, Clone)]
pub struct TileConfig {
    pub name: String,
    pub kind: TileKind,
    pub atlas: String,
    pub atlas_index: u16,
    pub map_color: HexColor,
    pub blend_tech: Option<BlendTech>,
}

#[derive(Asset, Default, Debug, Reflect, Clone)]
pub struct TileConfigList(pub Vec<TileConfig>);

impl ConfigParser for TileConfigList {
    type Config = Vec<TileConfig>;

    async fn from_config(
        config: Self::Config,
        _load_context: crate::loader::ConfigParserContext<'_, '_>,
    ) -> Result<Self, ConfigAssetLoaderError>
    where
        Self: Sized,
    {
        Ok(Self(config))
    }
}
