use bevy::prelude::*;

use crate::world::tile::TileType;

pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 512;

pub const SIZE: usize = WIDTH * HEIGHT;

#[derive(Debug)]
pub struct Grid<T> {
    pub data: Box<[T; SIZE]>,
}

impl<T> Default for Grid<T>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self {
            data: Box::new([T::default(); SIZE]),
        }
    }
}

#[derive(Default, Resource)]
pub struct Map {
    pub types: Grid<TileType>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            types: Default::default(),
        }
    }
}

