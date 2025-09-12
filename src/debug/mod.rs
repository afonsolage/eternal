use bevy::app::{App, Plugin};

mod ui_tilemap_chunk;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ui_tilemap_chunk::UITilemapChunkPlugin);
    }
}
