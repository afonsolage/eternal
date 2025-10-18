use crate::{atlas::Atlas, noise::Noise};

pub mod atlas;
mod noise;

pub fn generate_atlas() -> Atlas {
    let mut atlas = Atlas::new();
    let noise = Noise::new(42);
    for y in 0..atlas::DIMS.y {
        for x in 0..atlas::DIMS.x {
            atlas.elevation[atlas::to_index(x, y)] = noise.get(x as f32, y as f32);
        }
    }

    atlas
}
