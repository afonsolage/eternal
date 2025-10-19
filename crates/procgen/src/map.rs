use bevy::{ecs::resource::Resource, math::U16Vec2};

pub const MAP_AXIS_SIZE: usize = 256;
pub const MAP_SIZE: usize = MAP_AXIS_SIZE.pow(2);

pub fn to_index(x: u16, y: u16) -> usize {
    y as usize * MAP_AXIS_SIZE + x as usize
}

pub fn from_index(index: usize) -> U16Vec2 {
    U16Vec2 {
        x: (index % MAP_AXIS_SIZE) as u16,
        y: (index / MAP_AXIS_SIZE) as u16,
    }
}

#[derive(Default, Debug, Clone, Resource)]
pub struct Map {
    pub elevation: Vec<f32>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            elevation: vec![0.0; MAP_SIZE],
        }
    }
}
