use crate::{
    noise::Noise,
    world::{
        grid::{self, Grid, GridElevation, GridId, Layer, LayerIndex},
        tile::{TileElevation, TileId},
    },
};

pub fn generate_grids() -> (GridId, GridElevation) {
    let elevations = generate_elevation();
    let mut ids = Grid::new();

    collapse_floor_layer(&mut ids[LayerIndex::FLOOR], &elevations);
    collapse_wall_layer(&mut ids[LayerIndex::WALL], &elevations);

    (ids, elevations)
}

pub fn generate_elevation() -> GridElevation {
    let elevation_noise = Noise::new(42);

    let mut elevations = Grid::new();

    for y in 0..grid::DIMS.y as u16 {
        for x in 0..grid::DIMS.x as u16 {
            let elevation = elevation_noise.get(x as f32, y as f32);
            elevations.set(x, y, TileElevation::new(elevation));
        }
    }

    elevations
}

pub fn collapse_floor_layer(floor: &mut Layer<TileId>, elevations: &GridElevation) {
    for y in 0..grid::DIMS.y as u16 {
        for x in 0..grid::DIMS.x as u16 {
            let elevation = **elevations.get(x, y);

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

            floor.set(x, y, TileId::new(id));
        }
    }
}

pub fn collapse_wall_layer(wall: &mut Layer<TileId>, elevations: &GridElevation) {
    for y in 0..grid::DIMS.y as u16 {
        for x in 0..grid::DIMS.x as u16 {
            let elevation = **elevations.get(x, y);

            if elevation > 0.21 {
                wall.set(x, y, TileId::new(5));
            }
        }
    }
}
