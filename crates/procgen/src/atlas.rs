use bevy::{ecs::resource::Resource, math::U16Vec2};

pub const DIMS: U16Vec2 = U16Vec2::new(1024, 1024);

pub fn to_index(x: u16, y: u16) -> usize {
    y as usize * DIMS.x as usize + x as usize
}

pub fn from_index(index: usize) -> U16Vec2 {
    U16Vec2 {
        x: (index % DIMS.x as usize) as u16,
        y: (index / DIMS.x as usize) as u16,
    }
}

#[derive(Default, Debug, Clone, Resource)]
pub struct Atlas {
    pub elevation: Vec<f32>,
}

impl Atlas {
    pub fn new() -> Self {
        Self {
            elevation: vec![0.0; DIMS.as_usizevec2().element_product()],
        }
    }
}
