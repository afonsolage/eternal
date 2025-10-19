use bevy::log::debug;

use crate::{atlas::Atlas, noise::Noises};

pub mod atlas;
pub mod noise;

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
