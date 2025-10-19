use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use eternal_config::ConfigPlugin;
use eternal_grid::ecs::GridPlugin;
use eternal_procgen::ProcGenPlugin;

use crate::{atlas_editor::AtlasEditorPlugin, map_editor::MapEditorPlugin};

mod atlas_editor;
mod camera;
mod map_editor;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
            .add_plugins((
                ProcGenPlugin,
                camera::CameraPlugin,
                ConfigPlugin,
                GridPlugin,
            ))
            .add_plugins((MapEditorPlugin, AtlasEditorPlugin))
            .init_state::<EditorState>()
            .add_systems(Update, switch_editor_state);
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
enum EditorState {
    #[default]
    Atlas,
    Map,
}

fn switch_editor_state(
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<EditorState>>,
    mut next_state: ResMut<NextState<EditorState>>,
) {
    let current_state = *current_state.get();

    if input.just_released(KeyCode::Digit1) && current_state != EditorState::Atlas {
        debug!("Switching to Atlas Editor");
        next_state.set(EditorState::Atlas);
    } else if input.just_released(KeyCode::Digit2) && current_state != EditorState::Map {
        debug!("Switching to Map Editor");
        next_state.set(EditorState::Map);
    }
}
