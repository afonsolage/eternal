#![allow(unused)]
use std::ops::Deref;

use bevy::{
    asset::RenderAssetUsages,
    ecs::relationship::RelatedSpawnerCommands,
    image::ImageSampler,
    math::U16Vec2,
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
        grid::{self, Grid, GridId, GridVisible, Layer, LayerIndex},
        renderer::tilemap::Tilemap,
        tile::{self, TileId, TileRegistry},
    },
};
pub struct UIDrawTileMap;

impl Plugin for UIDrawTileMap {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_ui)
            .add_systems(Update, update_tile_map_color_ui.run_if(should_redraw_map));
    }
}

#[derive(Component)]
struct ImageContainer;

#[derive(Component)]
struct DisplayMapUI;

#[allow(clippy::type_complexity)]
fn should_redraw_map(
    registry: Res<TileRegistry>,
    q_grid_changed: Query<(), Or<(Changed<GridId>, Changed<GridVisible>)>>,
) -> bool {
    registry.is_changed() || !q_grid_changed.is_empty()
}

fn spawn_debug_ui(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = Image {
        data: Some(vec![0u8; grid::LAYER_SIZE * 4 * 4]), // 4 bytes per channel, 4 channels
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
    };

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
                        image: images.add(image),
                        ..Default::default()
                    }
                )],
            ),
        ),
        DisplayMapUI,
    ));
}

fn update_tile_map_color_ui(
    singleton: Single<(&GridId, &GridVisible)>,
    image_node: Single<&ImageNode, With<ImageContainer>>,
    tile_info: Res<TileRegistry>,
    mut images: ResMut<Assets<Image>>,
) {
    if tile_info.is_empty() {
        return;
    }

    let Some(mut image) = images.get_mut(image_node.image.id()) else {
        return;
    };

    let (grid_id, grid_visible) = singleton.into_inner();

    let visible_rect = grid_visible.calc_visibility_rect();

    draw_tile_map_colors(
        image.data.as_mut().expect("Data is initialized on setup"),
        grid_id,
        tile_info,
        visible_rect,
    );
}

fn draw_tile_map_colors(
    data: &mut [u8],
    grid: &GridId,
    tile_info_map: Res<TileRegistry>,
    visibility_rect: URect,
) {
    data.fill(0);

    let colors: &mut [[f32; 4]] = bytemuck::cast_slice_mut(data);

    let floor = &grid[LayerIndex::FLOOR];
    colors
        .iter_mut()
        .enumerate()
        .for_each(|(idx, color_buffer)| {
            let tile_id = floor[idx];

            let tile_info = tile_info_map.get(&tile_id).unwrap_or_else(|| {
                error!("No info found for tile id: {}", tile_id.deref());
                &tile::NONE_INFO
            });

            *color_buffer = tile_info.map_color.to_f32_array();
        });

    let mut set_color_at = |x: u32, y: u32, color: Color| {
        let index = grid::to_index(x as u16, y as u16);
        colors[index] = color.to_srgba().to_f32_array();
    };

    if !visibility_rect.is_empty() {
        for x in (visibility_rect.min.x..=visibility_rect.max.x) {
            set_color_at(x, visibility_rect.min.y, Color::WHITE);
            set_color_at(x, visibility_rect.max.y, Color::WHITE);
        }

        for y in (visibility_rect.min.y..=visibility_rect.max.y) {
            set_color_at(visibility_rect.min.x, y, Color::WHITE);
            set_color_at(visibility_rect.max.x, y, Color::WHITE);
        }
    }
}
