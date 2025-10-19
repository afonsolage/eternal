use bevy::{ecs::resource::Resource, math::U16Vec2};

pub const MAP_RESOLUTION: u16 = 3;
pub const MAP_COUNT: u16 = 128;
pub const ATLAS_AXIS_SIZE: usize = (MAP_COUNT * MAP_RESOLUTION) as usize;
pub const ATLAS_SIZE: usize = (ATLAS_AXIS_SIZE as usize).pow(2);

pub fn to_index(x: u16, y: u16) -> usize {
    y as usize * ATLAS_AXIS_SIZE + x as usize
}

pub fn from_index(index: usize) -> U16Vec2 {
    U16Vec2 {
        x: (index % ATLAS_AXIS_SIZE) as u16,
        y: (index / ATLAS_AXIS_SIZE) as u16,
    }
}

#[derive(Default, Debug, Clone, Resource)]
pub struct Atlas {
    pub elevation: Vec<f32>,
}

impl Atlas {
    pub fn new() -> Self {
        Self {
            elevation: vec![0.0; ATLAS_SIZE],
        }
    }
}
