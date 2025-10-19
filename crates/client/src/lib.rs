use bevy::prelude::*;

use eternal_config::ConfigPlugin;
use eternal_grid::grid;
use eternal_ui::UiPlugin;

use crate::{
    debug::DebugPlugin,
    effects::EffectsPlugin,
    player::{Player, PlayerPlugin},
    world::WorldPlugin,
};

mod debug;
mod effects;
mod noise;
mod player;
mod run_conditions;
mod world;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DebugPlugin)
            .add_plugins((
                EffectsPlugin,
                ConfigPlugin,
                WorldPlugin,
                PlayerPlugin,
                UiPlugin,
            ))
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Player,
        Transform::from_translation(grid::grid_to_world(140, 100).extend(0.0)),
    ));
}
