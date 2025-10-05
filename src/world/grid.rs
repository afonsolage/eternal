use bevy::prelude::*;

use crate::world::tile::{TileElevation, TileId, TileVisible};

pub const DIMS: UVec2 = UVec2::new(256, 256);
pub const LAYER_SIZE: usize = (DIMS.x * DIMS.y) as usize;

pub type GridId = Grid<TileId>;
pub type GridVisible = Grid<TileVisible>;
pub type GridElevation = Grid<TileElevation>;

#[derive(Debug, Default, Clone, Copy)]
#[repr(usize)]
pub enum LayerIndex {
    #[default]
    FLOOR,
    WALLS,
    ROOF,
}

#[derive(Default, Clone, Debug, Component)]
pub struct Grid<T, const N: usize = 1>(Vec<Layer<T>>);

fn to_index(x: u16, y: u16) -> usize {
    y as usize * DIMS.x as usize + x as usize
}

impl<T, const N: usize> Grid<T, N>
where
    T: Default + Clone,
{
    pub fn new() -> Self {
        Self(vec![Layer(vec![Default::default(); LAYER_SIZE]); N])
    }
}

impl<T, const N: usize> std::ops::Index<LayerIndex> for Grid<T, N> {
    type Output = Layer<T>;

    fn index(&self, index: LayerIndex) -> &Self::Output {
        let index = index as usize;
        debug_assert!(index < N);
        &self.0[index]
    }
}

impl<T, const N: usize> std::ops::IndexMut<LayerIndex> for Grid<T, N> {
    fn index_mut(&mut self, index: LayerIndex) -> &mut Self::Output {
        let index = index as usize;
        debug_assert!(index < N);
        &mut self.0[index]
    }
}

impl<T> std::ops::Index<usize> for Grid<T, 1> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[0][index]
    }
}

impl<T> std::ops::IndexMut<usize> for Grid<T, 1> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[0][index]
    }
}

impl<T> std::ops::Deref for Grid<T, 1> {
    type Target = Layer<T>;

    fn deref(&self) -> &Self::Target {
        &self.0[0]
    }
}

impl<T> std::ops::DerefMut for Grid<T, 1> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0[0]
    }
}

#[derive(Default, Clone, Debug, Deref, DerefMut)]
pub struct Layer<T>(Vec<T>);

impl<T> Layer<T> {
    pub fn get(&self, x: u16, y: u16) -> &T {
        &self[to_index(x, y)]
    }

    pub fn set(&mut self, x: u16, y: u16, value: T) {
        self[to_index(x, y)] = value
    }
}
