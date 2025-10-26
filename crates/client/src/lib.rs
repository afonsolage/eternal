use bevy::prelude::*;

use eternal_config::ConfigPlugin;
use eternal_grid::{ecs::TileRegistry, grid};
use eternal_procgen::biome::BiomeRegistry;
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
            .init_state::<ClientState>()
            .add_systems(OnEnter(ClientState::Playing), setup)
            .add_systems(Update, loading.run_if(in_state(ClientState::Loading)));
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
enum ClientState {
    #[default]
    Loading,
    Playing,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Player,
        Transform::from_translation(grid::grid_to_world(140, 100).extend(0.0)),
    ));
}

fn loading(
    biome_registry: Res<BiomeRegistry>,
    tile_registry: Res<TileRegistry>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    if !biome_registry.is_ready() || tile_registry.is_empty() {
        return;
    }

    next_state.set(ClientState::Playing);
}
