use bevy::{asset::AssetLoader, prelude::*};
use serde::Deserialize;

use crate::ConfigAssetLoaderError;

pub struct ConfigNoisePlugin;

impl Plugin for ConfigNoisePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<NoiseLayersConfig>()
            .init_asset_loader::<ConfigAssetLoader>();
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum NoiseKind {
    Simplex,
    Perlin,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Noise {
    Fbm {
        kind: NoiseKind,
        octaves: u8,
        frequency: f32,
        lacunarity: f32,
        gain: f32,
    },
    Ridge,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoiseLayer {
    pub name: String,
    pub noise: Noise,
}

#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct NoiseLayersConfig(pub Vec<NoiseLayer>);

#[derive(Default)]
struct ConfigAssetLoader;

impl AssetLoader for ConfigAssetLoader {
    type Asset = NoiseLayersConfig;

    type Settings = ();

    type Error = ConfigAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;

        use ron::extensions::Extensions;
        let opts = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);

        let tile_list = opts.from_bytes(&buffer)?;

        Ok(tile_list)
    }
}
