use std::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::{math::U16Vec2, prelude::*};

use crate::tile::{self, TileElevation, TileId, TileVisible};

pub const DIMS: UVec2 = UVec2::new(256, 256);
pub const LAYER_SIZE: usize = (DIMS.x * DIMS.y) as usize;

pub type GridId = Grid<TileId, { LAYERS.len() }>;
pub type GridVisible = Grid<TileVisible>;
pub type GridElevation = Grid<TileElevation>;

#[derive(Default, Event)]
pub struct GridIdChanged(pub LayerIndex, pub Vec<U16Vec2>);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum LayerIndex {
    #[default]
    Floor,
    Wall,
    Roof,
}

pub const LAYERS_COUNT: usize = 3;
pub const LAYERS: [LayerIndex; LAYERS_COUNT] =
    [LayerIndex::Floor, LayerIndex::Wall, LayerIndex::Roof];

impl LayerIndex {
    pub fn height(&self) -> f32 {
        match self {
            LayerIndex::Floor => -1.00,
            LayerIndex::Wall => 0.00,
            LayerIndex::Roof => 1.00,
        }
    }

    pub fn base_index(&self) -> usize {
        (*self as u32) as usize * LAYER_SIZE
    }
}

pub fn to_index(x: u16, y: u16) -> usize {
    y as usize * DIMS.x as usize + x as usize
}

pub fn grid_to_world(x: u16, y: u16) -> Vec2 {
    Vec2::new(x as f32, y as f32) * tile::SIZE.as_vec2()
}

#[derive(Default, Clone, Debug, Component)]
pub struct Grid<T, const N: usize = 1>(Vec<Layer<T>>);

impl<T, const N: usize> Grid<T, N>
where
    T: Default + Clone,
{
    pub fn new() -> Self {
        Self(vec![Layer::new(vec![Default::default(); LAYER_SIZE]); N])
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

#[derive(Debug)]
pub struct Layer<T> {
    data: Vec<T>,
    sender: Sender<(U16Vec2, T)>,
    // We need this Mutex since bevy ecs Component requires Send + Sync
    // I think there is a better way to do this.
    receiver: Mutex<Receiver<(U16Vec2, T)>>,
}

impl<T> Layer<T> {
    fn new(data: Vec<T>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            data,
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    pub fn get(&self, x: u16, y: u16) -> &T {
        &self.data[to_index(x, y)]
    }

    pub fn set(&mut self, x: u16, y: u16, value: T) {
        self.data[to_index(x, y)] = value;
    }

    pub fn positions(&self) -> impl Iterator<Item = (u16, u16, &T)> {
        self.iter()
            .enumerate()
            .map(|(i, t)| ((i as u32 % DIMS.x) as u16, (i as u32 / DIMS.x) as u16, t))
    }
}

impl<T> Layer<T> {
    pub fn queue(&self, x: u16, y: u16, value: T) {
        let _ = self.sender.send((U16Vec2::new(x, y), value));
    }

    pub fn drain_queue(&self) -> Vec<(U16Vec2, T)> {
        self.receiver
            .try_lock()
            .expect("Bevy ECS ensures only on exclusive access happens at any given time")
            .try_iter()
            .collect()
    }
}

impl<T> std::ops::Deref for Layer<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for Layer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> Default for Layer<T> {
    fn default() -> Self {
        Self::new(default())
    }
}

impl<T> Clone for Layer<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.data.clone())
    }
}

impl GridVisible {
    pub fn calc_visibility_rect(&self) -> URect {
        let mut rect = URect::EMPTY;

        self.positions()
            .filter_map(|(x, y, visible)| {
                if visible.is_visible() {
                    Some(UVec2::new(x as u32, y as u32))
                } else {
                    None
                }
            })
            .for_each(|pos| {
                if pos.x < rect.min.x {
                    rect.min.x = pos.x;
                } else if pos.x > rect.max.x {
                    rect.max.x = pos.x;
                }

                if pos.y < rect.min.y {
                    rect.min.y = pos.y;
                } else if pos.y > rect.max.y {
                    rect.max.y = pos.y;
                }
            });

        rect
    }
}
