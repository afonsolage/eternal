use crate::{
    noise::Noise,
    world::{
        grid::{self, Grid},
        tile::TileId,
    },
};

pub fn generate_tile_ids() -> Grid<TileId> {
    let noise = Noise::new();

    let mut tile_ids: Grid<TileId> = Grid::new();

    tile_ids
        .iter_mut()
        .enumerate()
        .map(|(idx, tile_type)| {
            // Row-Major
            let x = idx as i32 % grid::DIMS.x as i32;
            let y = idx as i32 / grid::DIMS.x as i32;
            (x, y, tile_type)
        })
        .for_each(|(x, y, tile_type)| {
            let i = noise.stone(x as f32, y as f32);

            let id = if i < -30 {
                2
            } else if i < -20 {
                4
            } else if i < -10 {
                0
            } else if i < 20 {
                1
            } else {
                3
            };

            let _ = std::mem::replace(tile_type, TileId::new(id));
        });

    tile_ids
}
