use bevy::prelude::*;
use eternal_grid::{
    grid::{self, LayerIndex},
    tile::TileElevation,
};

use crate::{
    atlas::{Atlas, AtlasPlugin},
    biome::{Biome, BiomePlugin},
    map::Map,
    noise::NoiseStack,
};

pub mod atlas;
pub mod biome;
pub mod map;
pub mod noise;

pub struct ProcGenPlugin;

impl Plugin for ProcGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BiomePlugin, AtlasPlugin));
    }
}

pub fn generate_atlas(noise_stack: &NoiseStack) -> Atlas {
    debug!("Generating atlas!");

    let mut atlas = Atlas::new();

    for y in 0..atlas::ATLAS_AXIS_SIZE as u16 {
        for x in 0..atlas::ATLAS_AXIS_SIZE as u16 {
            atlas.elevation[atlas::to_index(x, y)] = noise_stack.get(x as f32, y as f32);
        }
    }

    debug!("Atlas generated!");

    atlas
}

pub fn generate_map(biome: &Biome) -> Map {
    debug!("Generating map!");

    let mut map = Map::new(biome.name.clone());
    for y in 0..grid::DIMS.y as u16 {
        for x in 0..grid::DIMS.x as u16 {
            generate_terrain(x, y, biome, &mut map);
            generate_flora(x, y, biome, &mut map);
        }
    }

    debug!("Map generated!");

    map
}

fn generate_terrain(x: u16, y: u16, biome: &Biome, map: &mut Map) {
    let elevation = biome.terrain_noise.get(x as f32, y as f32);

    map.elevation.set(x, y, TileElevation::new(elevation));
    map.tile[LayerIndex::Floor].set(x, y, biome.terrain_pallet.collapse(elevation));
}

fn generate_flora(x: u16, y: u16, biome: &Biome, map: &mut Map) {
    let probability = biome.flora_noise.get(x as f32, y as f32);
    let elevation = **map.elevation.get(x, y);
    let tile = map.tile[LayerIndex::Floor].get(x, y);

    let Some(flora) = biome.flora_registry.iter().find(|flora| {
        flora.threshold > probability
            && (flora.allowed_terrains.is_empty() || flora.allowed_terrains.contains(tile))
            && flora
                .elevation_range
                .is_none_or(|(min, max)| elevation > min && elevation < max)
    }) else {
        return;
    };

    map.tile[LayerIndex::Wall].set(x, y, flora.tile);
}
