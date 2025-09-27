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
    T: Copy,
{
    pub fn try_get(&self, pos: IVec2) -> Option<T> {
        if pos.x < 0 || pos.x >= WIDTH as i32 || pos.y >= HEIGHT as i32 || pos.y < 0 {
            return None;
        }

        let index = pos.y as usize * WIDTH + pos.x as usize;

        Some(self.0[index])
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
