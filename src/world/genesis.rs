use crate::{
    noise::Noise,
    world::{
        grid::{self, Grid, GridElevation, GridId},
        tile::{TileElevation, TileId},
    },
};

pub fn generate_grids() -> (GridId, GridElevation) {
    let elevation_noise = Noise::new(42);

    let mut ids = Grid::new();
    let mut elevations = Grid::new();

    for y in 0..grid::DIMS.y {
        for x in 0..grid::DIMS.x {
            let elevation = elevation_noise.get(x as f32, y as f32);

            let id = if elevation < -0.3 {
                2
            } else if elevation < -0.2 {
                4
            } else if elevation < -0.1 {
                0
            } else if elevation < 0.2 {
                1
            } else {
                3
            };

            let index = (y * grid::DIMS.x + x) as usize;

            ids[0][index] = TileId::new(id);
            elevations[0][index] = TileElevation::new(elevation);
        }
    }

    (ids, elevations)
}
