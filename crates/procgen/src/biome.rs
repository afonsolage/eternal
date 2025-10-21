use bevy::prelude::*;
use eternal_config::{
    biome::{BiomePalletConfig, BiomeRegistryConfig},
    server::{ConfigAssetUpdated, ConfigServer, ConfigServerPlugin, Configs},
};
use eternal_grid::{ecs::TileRegistry, tile::TileId};

use crate::noise::NoiseStack;

pub(crate) struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ConfigServerPlugin::<BiomeRegistryConfig>::default(),
            ConfigServerPlugin::<BiomePalletConfig>::default(),
        ))
        .init_resource::<BiomeRegistry>()
        .add_systems(Startup, setup);
    }
}

#[derive(Default, Debug, Clone, Reflect)]
pub struct BiomePallet(Vec<(f32, TileId)>);

impl BiomePallet {
    pub fn collapse(&self, value: f32) -> TileId {
        for &(threshould, tile_id) in &self.0 {
            if value < threshould {
                return tile_id;
            }
        }

        Default::default()
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct Biome {
    pub name: String,
    pub terrain_noise: Handle<NoiseStack>,
    pub terrain_pallet: BiomePallet,
}

#[derive(Component, Deref)]
struct BiomeName(String);

#[derive(Default, Debug, Clone, Resource, Reflect, Deref)]
pub struct BiomeRegistry(Vec<Biome>);

impl BiomeRegistry {
    pub fn get_biome(&self, name: &str) -> Option<&Biome> {
        self.0.iter().find(|b| b.name == name)
    }

    pub fn get_biome_mut(&mut self, name: &str) -> Option<&mut Biome> {
        self.0.iter_mut().find(|b| b.name == name)
    }
}

fn setup(mut config_server: ConfigServer) {
    config_server
        .load::<BiomeRegistryConfig>("config/procgen/biomes.ron")
        .observe(on_biome_config_updated);
}

fn on_biome_config_updated(
    updated: On<ConfigAssetUpdated>,
    biome_configs: Configs<BiomeRegistryConfig>,
    asset_server: Res<AssetServer>,
    mut config_server: ConfigServer,
    mut commands: Commands,
) {
    let id = updated.id();
    let Some(BiomeRegistryConfig(biome_configs)) = biome_configs.get(id).cloned() else {
        error!("Biome registry config not found for id {}", id);
        return;
    };

    let mut biomes = Vec::with_capacity(biome_configs.len());

    for config in biome_configs {
        config_server
            .load::<BiomePalletConfig>(&config.pallet)
            .insert(BiomeName(config.name.clone()))
            .observe(on_biome_pallet_config_updated);
        let biome = Biome {
            name: config.name,
            terrain_noise: asset_server.load(&config.noise),
            terrain_pallet: Default::default(),
        };

        biomes.push(biome);
    }

    commands.insert_resource(BiomeRegistry(biomes));
}

fn on_biome_pallet_config_updated(
    updated: On<ConfigAssetUpdated>,
    q_names: Query<&BiomeName>,
    pallet_configs: Configs<BiomePalletConfig>,
    mut registry: ResMut<BiomeRegistry>,
    tile_registry: Res<TileRegistry>,
) {
    let Some(pallet_config) = pallet_configs.get(updated.id()) else {
        error!("Pallet config not found for id {}", updated.id());
        return;
    };

    let biome_name = q_names
        .get(updated.event_target())
        .expect("Observer to have BiomeName component")
        .as_str();

    let Some(biome) = registry.get_biome_mut(biome_name) else {
        error!("Biome {biome_name} not found on registry!");
        return;
    };

    debug!("Updating pallet of biome {}!", biome.name);

    let pallet = pallet_config
        .iter()
        .map(|(threshould, tile_name)| {
            let tile_id = tile_registry.get_id_by_name(tile_name);
            (*threshould, tile_id)
        })
        .collect();

    biome.terrain_pallet = BiomePallet(pallet);
}
