use bevy::prelude::*;

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
    let noise_fn = noise_stack.main();

    for y in 0..atlas::ATLAS_AXIS_SIZE as u16 {
        for x in 0..atlas::ATLAS_AXIS_SIZE as u16 {
            atlas.elevation[atlas::to_index(x, y)] = noise_fn.get([x as f64, y as f64]) as f32;
        }
    }

    debug!("Atlas generated!");

    atlas
}

pub fn generate_map(biome: &Biome) -> Map {
    debug!("Generating map!");

    let mut map = Map::new(biome.name.clone());
    let noise_fn = biome.terrain_noise.main();
    for y in 0..map::MAP_AXIS_SIZE as u16 {
        for x in 0..map::MAP_AXIS_SIZE as u16 {
            map.elevation[map::to_index(x, y)] = noise_fn.get([x as f64, y as f64]) as f32;
        }
    }

    debug!("Map generated!");

    map
}
