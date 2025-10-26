use bevy::prelude::*;

use crate::{
    color::HexColor,
    server::{ConfigServerPlugin, FromConfig},
};

pub(crate) struct TileConfigPlugin;
impl Plugin for TileConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ConfigServerPlugin::<TileConfigList>::default(),));
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
    Weight(u8),
}

#[derive(Debug, Reflect, Clone)]
pub struct TileConfig {
    pub name: String,
    pub kind: TileKind,
    pub atlas: String,
    pub atlas_index: u16,
    pub map_color: HexColor,
    pub outline: bool,
    pub blend_tech: Option<BlendTech>,
}

#[derive(Default, Debug, Reflect, Clone)]
pub struct TileConfigList(pub Vec<TileConfig>);

impl FromConfig for TileConfigList {
    type InnerType = Vec<TileConfig>;

    fn from_inner<'a>(asset: Self::InnerType) -> Self {
        Self(asset)
    }
}
