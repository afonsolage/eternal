use std::ops::Deref;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    math::U16Vec2,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};

use eternal_grid::{
    ecs::TileRegistry,
    grid::{self, GridId, GridIdChanged, GridVisible, LayerIndex},
    tile::{self},
};
use eternal_ui::window::{WindowConfig, window};

use crate::ClientState;
pub struct UIDrawTileMap;

impl Plugin for UIDrawTileMap {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientState::Playing), spawn_debug_ui)
            .add_systems(
                Update,
                (
                    update_whole_map
                        .run_if(resource_changed::<TileRegistry>.or(state_changed::<ClientState>)),
                    update_overlay.run_if(should_redraw_overlay),
                )
                    .run_if(in_state(ClientState::Playing)),
            )
            .add_observer(on_grid_id_changed);
    }
}

#[derive(Component)]
struct MapImage;

#[derive(Component)]
struct OverlayImage;

#[derive(Component)]
struct DisplayMapUI;

fn should_redraw_overlay(q: Query<(), Changed<GridVisible>>) -> bool {
    !q.is_empty()
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
                children![
                    (
                        Name::new("Map"),
                        MapImage,
                        ImageNode {
                            flip_y: true,
                            image: images.add(image.clone()),
                            ..Default::default()
                        }
                    ),
                    (
                        Name::new("Overlay"),
                        OverlayImage,
                        ImageNode {
                            flip_y: true,
                            image: images.add(image),
                            ..Default::default()
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(100.0),
                            height: percent(100.0),
                            ..default()
                        },
                    )
                ],
            ),
        ),
        DisplayMapUI,
    ));
}

fn on_grid_id_changed(
    changed: On<GridIdChanged>,
    grid: Single<&GridId>,
    image_node: Single<&ImageNode, With<MapImage>>,
    tile_info: Res<TileRegistry>,
    mut images: ResMut<Assets<Image>>,
) {
    if tile_info.is_empty() {
        return;
    }

    let GridIdChanged(layer, positions) = &*changed;

    if !matches!(layer, LayerIndex::Floor) {
        return;
    }

    let Some(image) = images.get_mut(image_node.image.id()) else {
        return;
    };

    let floor = &grid[LayerIndex::Floor];
    for &U16Vec2 { x, y } in positions {
        let tile_id = floor.get(x, y);

        let tile_info = tile_info.get(tile_id).unwrap_or_else(|| {
            error!("No info found for tile id: {}", tile_id.deref());
            &tile::NONE_INFO
        });

        let _ = image.set_color_at(x as u32, y as u32, Color::Srgba(tile_info.map_color));
    }
}

fn update_whole_map(
    grid: Single<&GridId>,
    image_node: Single<&ImageNode, With<MapImage>>,
    tile_info: Res<TileRegistry>,
    mut images: ResMut<Assets<Image>>,
) {
    if tile_info.is_empty() {
        return;
    }

    let Some(image) = images.get_mut(image_node.image.id()) else {
        return;
    };

    let data = image.data.as_mut().expect("Data is initialized on setup");

    data.fill(0);

    let colors: &mut [[f32; 4]] = bytemuck::cast_slice_mut(data);

    let floor = &grid[LayerIndex::Floor];
    colors
        .iter_mut()
        .enumerate()
        .for_each(|(idx, color_buffer)| {
            let tile_id = floor[idx];

            let tile_info = tile_info.get(&tile_id).unwrap_or_else(|| {
                error!("No info found for tile id: {}", tile_id.deref());
                &tile::NONE_INFO
            });

            *color_buffer = tile_info.map_color.to_f32_array();
        });
}

fn update_overlay(
    grid_visible: Single<&GridVisible>,
    image_node: Single<&ImageNode, With<OverlayImage>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(image) = images.get_mut(image_node.image.id()) else {
        return;
    };

    let visible_rect = grid_visible.calc_visibility_rect();

    draw_overlay(
        image.data.as_mut().expect("Data is initialized on setup"),
        visible_rect,
    );
}

fn draw_overlay(data: &mut [u8], visibility_rect: URect) {
    data.fill(0);

    let colors: &mut [[f32; 4]] = bytemuck::cast_slice_mut(data);

    let mut set_color_at = |x: u32, y: u32, color: Color| {
        let index = grid::to_index(x as u16, y as u16);
        colors[index] = color.to_srgba().to_f32_array();
    };

    if !visibility_rect.is_empty() {
        for x in visibility_rect.min.x..=visibility_rect.max.x {
            set_color_at(x, visibility_rect.min.y, Color::WHITE);
            set_color_at(x, visibility_rect.max.y, Color::WHITE);
        }

        for y in visibility_rect.min.y..=visibility_rect.max.y {
            set_color_at(visibility_rect.min.x, y, Color::WHITE);
            set_color_at(visibility_rect.max.x, y, Color::WHITE);
        }
    }
}
