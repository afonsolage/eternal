use crate::server::{ConfigServerPlugin, FromConfig};
use bevy::prelude::*;

pub(crate) struct BiomeConfigPlugin;
impl Plugin for BiomeConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ConfigServerPlugin::<BiomeRegistryConfig>::default(),
            ConfigServerPlugin::<BiomePalletConfig>::default(),
        ));
    }
}

#[derive(Reflect, Default, Debug, Clone)]
pub struct BiomeConfig {
    pub name: String,
    pub terrain_noise: String,
    pub terrain_pallet: String,
    pub flora: String,
    pub flora_noise: String,
}

#[derive(Reflect, Default, Debug, Clone, Deref)]
pub struct BiomeRegistryConfig(pub Vec<BiomeConfig>);

impl FromConfig for BiomeRegistryConfig {
    type InnerType = Vec<(String, String)>;

    fn from_inner(asset: Self::InnerType) -> Self {
        Self(
            asset
                .into_iter()
                .map(|(name, path)| BiomeConfig {
                    name,
                    terrain_noise: format!("config/procgen/{path}/terrain_noise.ron"),
                    terrain_pallet: format!("config/procgen/{path}/terrain_pallet.ron"),
                    flora: format!("config/procgen/{path}/flora.ron"),
                    flora_noise: format!("config/procgen/{path}/flora_noise.ron"),
                })
                .collect(),
        )
    }
}

#[derive(Reflect, Default, Debug, Clone, Deref)]
pub struct BiomePalletConfig(Vec<(f32, String)>);

impl FromConfig for BiomePalletConfig {
    type InnerType = Vec<(f32, String)>;

    fn from_inner(asset: Self::InnerType) -> Self {
        Self(asset)
    }
}
