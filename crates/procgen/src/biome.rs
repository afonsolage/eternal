use bevy::prelude::*;
use eternal_config::{
    biome::{BiomePalletConfig, BiomeRegistryConfig},
    flora::FloraRegistryConfig,
    noise::NoiseStackConfig,
    server::{ConfigAssetUpdated, ConfigServer, Configs},
};
use eternal_grid::{ecs::TileRegistry, grid::LayerIndex, tile::TileId};

use crate::noise::NoiseStack;

pub(crate) struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BiomeRegistry>()
            .add_systems(Startup, setup);
    }
}

#[derive(Default, Debug, Clone, Reflect)]
pub struct Flora {
    pub name: String,
    pub tile: TileId,
    pub threshold: f32,
    pub wall_spacing: u8,
    pub floor_spacing: u8,
    pub elevation_range: Option<(f32, f32)>,
    pub allowed_terrains: Vec<TileId>,
}

#[derive(Default, Debug, Clone, Reflect, Deref)]
pub struct FloraRegistry(Vec<Flora>);

#[derive(Default, Debug, Clone, Reflect)]
pub struct BiomePallet {
    floor: Vec<(f32, TileId)>,
    wall: Vec<(f32, TileId)>,
}

impl BiomePallet {
    pub fn collapse(&self, layer: LayerIndex, value: f32) -> TileId {
        let layer = match layer {
            LayerIndex::Floor => &self.floor,
            LayerIndex::Wall => &self.wall,
            LayerIndex::Roof => todo!(),
        };

        for &(threshould, tile_id) in layer {
            if value < threshould {
                return tile_id;
            }
        }

        Default::default()
    }

    fn is_ready(&self) -> bool {
        !self.floor.is_empty()
    }
}

#[derive(Default, Debug, Clone, Reflect)]
pub struct Biome {
    pub name: String,
    pub flora_registry: FloraRegistry,
    pub flora_noise: NoiseStack,
    pub terrain_noise: NoiseStack,
    pub terrain_pallet: BiomePallet,
}

impl Biome {
    fn is_ready(&self) -> bool {
        self.flora_noise.is_ready()
            && self.terrain_noise.is_ready()
            && self.terrain_pallet.is_ready()
            && !self.flora_registry.is_empty()
    }
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

    pub fn is_ready(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        self.iter().all(|b| b.is_ready())
    }
}

fn setup(mut config_server: ConfigServer) {
    config_server
        .load::<BiomeRegistryConfig>("config/procgen/biomes.ron")
        .observe(on_biome_config_updated);
}

#[derive(Component, Debug, Clone, Copy)]
enum NoiseType {
    Terrain,
    Flora,
}

fn on_biome_config_updated(
    updated: On<ConfigAssetUpdated>,
    biome_configs: Configs<BiomeRegistryConfig>,
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
            .load::<BiomePalletConfig>(&config.terrain_pallet)
            .insert(BiomeName(config.name.clone()))
            .observe(on_biome_terrain_pallet_config_updated);

        config_server
            .load::<NoiseStackConfig>(&config.terrain_noise)
            .insert((BiomeName(config.name.clone()), NoiseType::Terrain))
            .observe(on_biome_noise_config_updated);

        config_server
            .load::<NoiseStackConfig>(&config.flora_noise)
            .insert((BiomeName(config.name.clone()), NoiseType::Flora))
            .observe(on_biome_noise_config_updated);

        config_server
            .load::<FloraRegistryConfig>(&config.flora)
            .insert(BiomeName(config.name.clone()))
            .observe(on_biome_flora_config_updated);

        let biome = Biome {
            name: config.name,
            ..default()
        };

        biomes.push(biome);
    }

    commands.insert_resource(BiomeRegistry(biomes));
}

fn on_biome_terrain_pallet_config_updated(
    updated: On<ConfigAssetUpdated>,
    q_names: Query<&BiomeName>,
    pallet_configs: Configs<BiomePalletConfig>,
    mut registry: ResMut<BiomeRegistry>,
    tile_registry: Res<TileRegistry>,
) {
    let biome_name = q_names
        .get(updated.event_target())
        .expect("Observer to have BiomeName component")
        .as_str();

    debug!("Updating pallet of biome {biome_name}!");

    let Some(pallet_config) = pallet_configs.get(updated.id()) else {
        error!("Pallet config not found for biome {biome_name}");
        return;
    };

    let Some(biome) = registry.get_biome_mut(biome_name) else {
        error!("Biome {biome_name} not found on registry!");
        return;
    };

    let floor = pallet_config
        .floor
        .iter()
        .map(|(threshould, tile_name)| {
            let tile_id = tile_registry.get_id_by_name(tile_name);
            (*threshould, tile_id)
        })
        .collect();

    let wall = pallet_config
        .wall
        .iter()
        .map(|(threshould, tile_name)| {
            let tile_id = tile_registry.get_id_by_name(tile_name);
            (*threshould, tile_id)
        })
        .collect();

    biome.terrain_pallet = BiomePallet { floor, wall }
}

fn on_biome_noise_config_updated(
    updated: On<ConfigAssetUpdated>,
    q_params: Query<(&BiomeName, &NoiseType)>,
    configs: Configs<NoiseStackConfig>,
    mut registry: ResMut<BiomeRegistry>,
) {
    let (BiomeName(biome_name), &noise_type) = q_params
        .get(updated.event_target())
        .expect("Observer to have BiomeName and NoiseType components");

    debug!("Updating noise of biome {biome_name} ({noise_type:?})");

    let Some(noise_config) = configs.get(updated.id()) else {
        error!("Noise config not found for biome {biome_name}.");
        return;
    };

    let stack = match NoiseStack::from_config(noise_config) {
        Ok(s) => s,
        Err(err) => {
            error!("Failed to update terrain noise for biome {biome_name}. {err}");
            return;
        }
    };

    let Some(biome) = registry.get_biome_mut(biome_name) else {
        error!("Biome {biome_name} not found on registry!");
        return;
    };

    match noise_type {
        NoiseType::Terrain => biome.terrain_noise = stack,
        NoiseType::Flora => biome.flora_noise = stack,
    }
}

fn on_biome_flora_config_updated(
    updated: On<ConfigAssetUpdated>,
    q_names: Query<&BiomeName>,
    flora_configs: Configs<FloraRegistryConfig>,
    mut registry: ResMut<BiomeRegistry>,
    tile_registry: Res<TileRegistry>,
) {
    let biome_name = q_names
        .get(updated.event_target())
        .expect("Observer to have BiomeName component")
        .as_str();

    debug!("Updating flora registry of biome {biome_name}!");

    let Some(flora_registry_config) = flora_configs.get(updated.id()) else {
        error!("Flora registry config not found for biome {biome_name}");
        return;
    };

    let Some(biome) = registry.get_biome_mut(biome_name) else {
        error!("Biome {biome_name} not found on registry!");
        return;
    };

    let registry = flora_registry_config
        .iter()
        .map(|flora_config| Flora {
            name: flora_config.name.clone(),
            tile: tile_registry.get_id_by_name(&flora_config.tile),
            threshold: flora_config.threshold,
            wall_spacing: flora_config.wall_spacing,
            floor_spacing: flora_config.floor_spacing,
            elevation_range: flora_config.elevation_range,
            allowed_terrains: flora_config
                .allowed_terrains
                .iter()
                .map(|name| tile_registry.get_id_by_name(name))
                .collect(),
        })
        .collect();

    biome.flora_registry = FloraRegistry(registry);
}
