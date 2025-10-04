use bevy::prelude::*;

use crate::world::tile::{TileElevation, TileId, TileVisible};

pub const DIMS: UVec2 = UVec2::new(256, 256);
pub const LAYER_SIZE: usize = (DIMS.x * DIMS.y) as usize;

pub type GridId = Grid<TileId>;
pub type GridVisible = Grid<TileVisible>;
pub type GridElevation = Grid<TileElevation>;

#[derive(Default, Clone, Debug, Reflect, Component)]
pub struct Grid<T, const N: usize = 1>(Vec<T>);

impl<T, const N: usize> Grid<T, N> {
    #[inline]
    fn to_index(x: u16, y: u16, layer: usize) -> usize {
        layer * LAYER_SIZE + y as usize * DIMS.x as usize + x as usize
    }

    pub fn layer(&self, layer: usize) -> &[T] {
        debug_assert!(layer < N, "Invalid layer");

        let begin = layer * LAYER_SIZE;
        let end = begin + LAYER_SIZE;
        self.0.get(begin..end).expect("Layer must be valid")
    }

    pub fn layer_mut(&mut self, layer: usize) -> &mut [T] {
        debug_assert!(layer < N, "Invalid layer");

        let begin = layer * LAYER_SIZE;
        let end = begin + LAYER_SIZE;
        self.0.get_mut(begin..end).expect("Layer must be valid")
    }
}

impl<T, const N: usize> Grid<T, N>
where
    T: Default + Clone,
{
    pub fn new() -> Self {
        Self(vec![
            Default::default();
            DIMS.extend(N as u32).element_product() as usize
        ])
    }

    pub fn set(&mut self, x: u16, y: u16, layer: usize, value: T) {
        self.0[Self::to_index(x, y, layer)] = value;
    }

    pub fn get(&self, x: u16, y: u16, layer: usize) -> &T {
        &self.0[Self::to_index(x, y, layer)]
    }
}

impl<T> std::ops::Index<usize> for Grid<T, 1usize> {
    type Output = [T];

    fn index(&self, layer: usize) -> &Self::Output {
        self.layer(layer)
    }
}

impl<T> std::ops::IndexMut<usize> for Grid<T, 1usize> {
    fn index_mut(&mut self, layer: usize) -> &mut Self::Output {
        self.layer_mut(layer)
    }
}
