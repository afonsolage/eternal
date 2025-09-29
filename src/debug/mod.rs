use bevy::app::{App, Plugin};

mod draw_grids;
mod ui_tile_map;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ui_tile_map::UIDrawTileMap, draw_grids::DrawGridsPlugin));
    }
}
