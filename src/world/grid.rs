use bevy::prelude::*;

pub const DIMS: UVec2 = UVec2::new(256, 256);

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
        Self(vec![Default::default(); DIMS.element_product() as usize])
    }
}
