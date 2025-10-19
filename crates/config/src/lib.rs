use bevy::prelude::*;
use thiserror::Error;

pub mod color;
pub mod noise;
pub mod tile;
use tile::ConfigTilePlugin;

use crate::noise::ConfigNoisePlugin;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ConfigTilePlugin, ConfigNoisePlugin));
    }
}

#[derive(Debug, Error)]
enum ConfigAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}
