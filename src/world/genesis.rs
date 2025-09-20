use crate::{
    noise::Noise,
    world::map::{self, Map},
};

pub fn generate_new_map() -> Map {
    let noise = Noise::new();

    let mut map = Map::new();

    map.types
        .data
        .iter_mut()
        .enumerate()
        .map(|(idx, tile_type)| {
            // Row-Major
            let x = (idx % map::HEIGHT) as i32;
            let y = (idx / map::HEIGHT) as i32;
            (x, y, tile_type)
        })
        .for_each(|(x, y, tile_type)| {
            let i = noise.stone(x as f32, y as f32);
            let tt = if i > 0 { 13 } else { 5 };
            tile_type.0 = tt as u16;
        });

    map
}
