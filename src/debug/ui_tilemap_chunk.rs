#![allow(unused)]
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

use crate::{
    tilemap::{Tilemap, TilemapChunkMap},
    ui::window::spawn_window,
};

pub struct UITilemapChunkPlugin;

impl Plugin for UITilemapChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_ui);
    }
}

#[derive(Component)]
struct DebugUIRoot;

fn spawn_debug_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_window(
        &mut commands,
        &asset_server,
        "[Debug] Tilemap Chunks",
        DebugUIRoot,
    )
    .with_children(|parent| {
        parent.spawn(Text::new("LoL"));
    });
}
