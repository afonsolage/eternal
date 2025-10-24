use bevy::prelude::*;
use thiserror::Error;

use crate::{
    biome::BiomeConfigPlugin, flora::FloraConfigPlugin, noise::NoiseStackConfigPlugin,
    tile::TileConfigPlugin,
};

pub mod biome;
pub mod color;
pub mod flora;
pub mod noise;
pub mod server;
pub mod tile;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BiomeConfigPlugin,
            FloraConfigPlugin,
            TileConfigPlugin,
            NoiseStackConfigPlugin,
        ));
    }
}

#[derive(Debug, Error)]
pub enum ConfigAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonError(#[from] ron::error::Error),
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error("Failed to read asset: {0}")]
    ReadAssetError(#[from] bevy::asset::ReadAssetBytesError),
    #[error("Failed to deserialize asset. Reflect Error: {0}")]
    Reflect(String),
    #[error("Failed to load asset: {0}")]
    Error(Box<dyn std::error::Error + Send + Sync + 'static>),
}
