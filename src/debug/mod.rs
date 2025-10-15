use bevy::{
    app::{App, Plugin},
    dev_tools::fps_overlay::FpsOverlayPlugin,
};

mod camera;
mod ui_settings;
mod ui_tile_map;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsOverlayPlugin::default()).add_plugins((
            ui_tile_map::UIDrawTileMap,
            ui_settings::UiDebugSettingsPlugin,
            camera::DebugCameraPlugin,
        ));
    }
}
