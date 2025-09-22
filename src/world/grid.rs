use bevy::prelude::*;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 256;

pub const SIZE: usize = WIDTH * HEIGHT;

#[derive(Default, Debug, Reflect, Deref, DerefMut, Component)]
pub struct Grid<T>(Vec<T>);

impl<T> Grid<T> {
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> Grid<T>
where
    T: Default + Clone,
{
    pub fn new() -> Self {
        Self(vec![Default::default(); SIZE])
    }
}
