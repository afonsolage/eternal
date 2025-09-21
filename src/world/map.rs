use bevy::prelude::*;

use crate::world::tile::TileId;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 256;

pub const SIZE: usize = WIDTH * HEIGHT;

#[derive(Default, Debug, Reflect)]
pub struct Grid<T> {
    pub data: Vec<T>,
}

#[derive(Debug, Default, Reflect)]
pub struct Map {
    pub types: Grid<TileId>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            types: Grid {
                data: vec![Default::default(); SIZE],
            },
        }
    }
}
