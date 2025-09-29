use bevy::prelude::*;

pub const DIMS: UVec2 = UVec2::new(256, 256);

#[derive(Default, Debug, Reflect, Deref, DerefMut, Component)]
pub struct Grid<T>(Vec<T>);

impl<T> Grid<T> {
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self(vec)
    }

    pub fn get(&self, x: u16, y: u16) -> &T {
        &self.0[Self::to_index(x, y)]
    }

    pub fn set(&mut self, x: u16, y: u16, value: T) {
        self.0[Self::to_index(x, y)] = value;
    }

    #[inline]
    fn to_index(x: u16, y: u16) -> usize {
        (y * DIMS.x as u16 + x) as usize
    }
}

impl<T> Grid<T>
where
    T: Default + Clone,
{
    pub fn new() -> Self {
        Self(vec![Default::default(); DIMS.element_product() as usize])
    }
}
