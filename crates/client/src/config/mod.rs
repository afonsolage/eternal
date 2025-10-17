use bevy::prelude::*;
use thiserror::Error;

use crate::config::tile::ConfigTilePlugin;

pub mod color;
pub mod tile;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConfigTilePlugin);
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
