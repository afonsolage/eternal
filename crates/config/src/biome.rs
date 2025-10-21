use crate::server::FromConfig;
use bevy::prelude::*;

#[derive(Reflect, Default, Debug, Clone)]
pub struct BiomeConfig {
    pub name: String,
    pub noise: String,
    pub pallet: String,
}

#[derive(Reflect, Default, Debug, Clone, Deref)]
pub struct BiomeRegistryConfig(pub Vec<BiomeConfig>);

impl FromConfig for BiomeRegistryConfig {
    type InnerType = Vec<(String, String)>;

    fn from_inner<'a, 'ctx>(
        asset: Self::InnerType,
        _load_context: &'a mut bevy::asset::LoadContext<'ctx>,
    ) -> Self {
        Self(
            asset
                .into_iter()
                .map(|(name, path)| BiomeConfig {
                    name,
                    noise: format!("config/procgen/{path}/terrain.ron"),
                    pallet: format!("config/procgen/{path}/pallet.ron"),
                })
                .collect(),
        )
    }
}

#[derive(Reflect, Default, Debug, Clone, Deref)]
pub struct BiomePalletConfig(Vec<(f32, String)>);

impl FromConfig for BiomePalletConfig {
    type InnerType = Vec<(f32, String)>;

    fn from_inner<'a, 'ctx>(
        asset: Self::InnerType,
        _load_context: &'a mut bevy::asset::LoadContext<'ctx>,
    ) -> Self {
        Self(asset)
    }
}
