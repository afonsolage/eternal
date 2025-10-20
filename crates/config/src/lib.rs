use bevy::prelude::*;
use thiserror::Error;

pub mod color;
pub mod loader;
pub mod noise;
pub mod tile;
use tile::ConfigTilePlugin;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConfigTilePlugin);
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
    #[error("Failed to deserialize asset. Reflect Error")]
    Reflect,
    #[error("Failed to load asset: {0}")]
    Error(Box<dyn std::error::Error + Send + Sync + 'static>),
}
