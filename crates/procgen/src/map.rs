use bevy::ecs::resource::Resource;
use eternal_grid::grid::{Grid, GridElevation, GridId};

#[derive(Default, Debug, Clone, Resource)]
pub struct Map {
    pub biome: String,
    pub elevation: GridElevation,
    pub tile: GridId,
}

impl Map {
    pub fn new(biome: String) -> Self {
        Self {
            elevation: Grid::new(),
            biome,
            tile: GridId::new(),
        }
    }
}
