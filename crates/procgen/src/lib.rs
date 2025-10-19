use bevy::prelude::*;

use crate::{
    atlas::Atlas,
    biome::{Biome, BiomePlugin},
    map::Map,
    noise::{NoisePlugin, Noises},
};

pub mod atlas;
pub mod biome;
pub mod map;
pub mod noise;

pub struct ProcGenPlugin;

impl Plugin for ProcGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((NoisePlugin, BiomePlugin));
    }
}

pub fn generate_atlas(noises: &Noises) -> Atlas {
    debug!("Generating atlas!");

    let mut atlas = Atlas::new();
    let noise_fn = noises.atlas();

    for y in 0..atlas::ATLAS_AXIS_SIZE as u16 {
        for x in 0..atlas::ATLAS_AXIS_SIZE as u16 {
            atlas.elevation[atlas::to_index(x, y)] = noise_fn.get([x as f64, y as f64]) as f32;
        }
    }

    debug!("Atlas generated!");

    atlas
}

pub fn generate_map(noises: &Noises, biome: &Biome) -> Map {
    debug!("Generating map!");

    let mut map = Map::new(biome.name.clone());
    if let Some(noise_fn) = noises.get_noise(biome.terrain_noise.id()) {
        for y in 0..map::MAP_AXIS_SIZE as u16 {
            for x in 0..map::MAP_AXIS_SIZE as u16 {
                map.elevation[map::to_index(x, y)] = noise_fn.get([x as f64, y as f64]) as f32;
            }
        }

        debug!("Map generated!");
    } else {
        debug!("Noise stack not found for biome {}", biome.name);
    }

    map
}
