#![allow(unused)]
use bevy::{math::U16Vec2, prelude::*};

use crate::world::renderer::tilemap::TilemapPlugin;

pub mod tilemap;

pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin).add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    //
}
