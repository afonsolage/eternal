use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        query::With,
        system::{Query, Res},
    },
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        Style, Val, JustifyContent, AlignItems, FlexDirection, UiRect
    },
    text::{Text, TextStyle},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    prelude::{Component, Name},
    hierarchy::BuildChildren
};

use crate::tilemap::{Tilemap, TilemapChunkMap};

pub struct UITilemapChunkPlugin;

impl Plugin for UITilemapChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_debug_ui);
    }
}

#[derive(Component)]
struct DebugUIRoot;

fn spawn_debug_ui(mut commands: Commands, q_tilemaps: Query<(&Tilemap, &TilemapChunkMap)>, q_ui_root: Query<Entity, With<DebugUIRoot>>) {
    // Check if the UI has already been spawned
    if q_ui_root.iter().next().is_some() {
        // Clear existing UI
        for entity in q_ui_root.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Spawn the root UI node
    let root_entity = commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        },
        DebugUIRoot,
        Name::new("Debug UI")
    )).id();

    for (tilemap, chunk_map) in q_tilemaps.iter() {
        for (chunk_pos, chunk) in chunk_map.iter() {
            let chunk_text = format!("Chunk: {:?}", chunk_pos);
            let mut tiles_text = "Tiles: ".to_string();

            for (i, tile) in chunk.tiles.iter().enumerate() {
                if tile.is_some() {
                    tiles_text.push_str(&format!("{}, ", i));
                }
            }

            let text_style = TextStyle {
                font_size: 14.0,
                ..Default::default()
            };

            let chunk_entity = commands.spawn(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..Default::default()
                },
                ..Default::default()
            }).id();

            let chunk_text_entity = commands.spawn(TextBundle::from_section(chunk_text, text_style.clone())).id();
            let tiles_text_entity = commands.spawn(TextBundle::from_section(tiles_text, text_style.clone())).id();

            commands.entity(chunk_entity).push_children(&[chunk_text_entity, tiles_text_entity]);
            commands.entity(root_entity).push_child(chunk_entity);
        }
    }
}
