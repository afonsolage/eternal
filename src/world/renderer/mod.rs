use bevy::prelude::*;

use crate::world::renderer::tilemap::TilemapPlugin;

pub mod tilemap;

pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);
    }
}
