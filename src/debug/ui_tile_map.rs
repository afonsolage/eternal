#![allow(unused)]
use std::ops::Deref;

use bevy::{
    asset::RenderAssetUsages,
    ecs::relationship::RelatedSpawnerCommands,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    ui_widgets::observe,
};

use crate::{
    config::tile::TileConfigList,
    ui::window::{WindowConfig, window},
    world::{
        grid::{self, Grid, GridId, LayerIndex},
        renderer::tilemap::Tilemap,
        tile::{self, TileId, TileRegistry},
    },
};
pub struct UIDrawTileMap;

impl Plugin for UIDrawTileMap {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_ui).add_systems(
            Update,
            update_tile_map_color_ui.run_if(
                resource_exists::<TileRegistry>.and(
                    resource_changed::<TileRegistry>
                        .or(|q: Query<&Grid<TileId>, Changed<Grid<TileId>>>| !q.is_empty()),
                ),
            ),
        );
    }
}

#[derive(Component)]
struct ImageContainer;

#[derive(Component)]
struct DisplayMapUI;

fn spawn_debug_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug Map"),
        window(
            WindowConfig {
                title: "[Debug] Map".to_string(),
                top: px(1.0),
                right: px(1.0),
                ..default()
            },
            (
                Name::new("Image Container"),
                Node {
                    width: percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                children![(
                    ImageContainer,
                    ImageNode {
                        flip_y: true,
                        ..Default::default()
                    }
                )],
            ),
        ),
        DisplayMapUI,
    ));
}

fn update_tile_map_color_ui(
    grid: Single<&GridId>,
    mut q_containers: Query<&mut ImageNode, With<ImageContainer>>,
    tile_info: Res<TileRegistry>,
    mut images: ResMut<Assets<Image>>,
    ui_entity: Local<Option<Entity>>,
    mut commands: Commands,
) {
    let Ok(mut image_node) = q_containers.single_mut() else {
        return;
    };

    debug!("Updating map");

    image_node.image = images.add(draw_tile_map_colors(&grid, tile_info));
}

fn draw_tile_map_colors(grid: &GridId, tile_info_map: Res<TileRegistry>) -> Image {
    let data = grid[LayerIndex::FLOOR]
        .iter()
        .filter_map(|id| {
            tile_info_map.get(id).or_else(|| {
                error!("No info found for tile id: {}", id.deref());
                Some(&tile::NONE_INFO)
            })
        })
        .flat_map(|tile_info| {
            tile_info
                .map_color
                .to_f32_array()
                .into_iter()
                .flat_map(|f| f.to_le_bytes())
        })
        .collect::<Vec<_>>();

    Image {
        data: Some(data),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: grid::DIMS.x,
                height: grid::DIMS.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler: ImageSampler::nearest(),
        asset_usage: RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        ..Default::default()
    }
}
