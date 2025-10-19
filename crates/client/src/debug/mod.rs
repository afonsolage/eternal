use bevy::app::{App, Plugin};

pub mod camera;
mod diagnostics;
mod inspector;
mod ui_settings;
mod ui_tile_map;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            inspector::InspectorPlugin,
            ui_tile_map::UIDrawTileMap,
            ui_settings::UiDebugSettingsPlugin,
            camera::DebugCameraPlugin,
            diagnostics::DiagnosticsPlugin,
        ));
    }
}
