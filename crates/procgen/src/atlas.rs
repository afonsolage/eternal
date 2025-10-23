use bevy::{math::U16Vec2, prelude::*};
use eternal_config::{
    noise::NoiseStackConfig,
    server::{ConfigAssetUpdated, ConfigServer, Configs},
};

use crate::noise::NoiseStack;

pub const MAP_RESOLUTION: u16 = 3;
pub const MAP_COUNT: u16 = 128;
pub const ATLAS_AXIS_SIZE: usize = (MAP_COUNT * MAP_RESOLUTION) as usize;
pub const ATLAS_SIZE: usize = ATLAS_AXIS_SIZE.pow(2);

pub(crate) struct AtlasPlugin;

impl Plugin for AtlasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut config_server: ConfigServer) {
    config_server
        .load::<NoiseStackConfig>("config/procgen/atlas.ron")
        .observe(on_noise_stack_config_updated);
}

fn on_noise_stack_config_updated(
    updated: On<ConfigAssetUpdated>,
    configs: Configs<NoiseStackConfig>,
    mut commands: Commands,
) {
    let Some(config) = configs.get(updated.id()) else {
        error!("Failed to get atlas noise config.");
        return;
    };

    let stack = match NoiseStack::from_config(config) {
        Ok(s) => s,
        Err(err) => {
            error!("Failed to update noise stack config. {err}");
            return;
        }
    };

    let atlas = crate::generate_atlas(&stack);

    commands.insert_resource(atlas);
}

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
