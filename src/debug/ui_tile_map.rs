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
};

use crate::{
    config::tile::TileConfigList,
    ui::window::spawn_window,
    world::{
        grid::{self, Grid},
        renderer::tilemap::Tilemap,
        tile::{self, TileId, TileInfos},
    },
};

pub struct UIDrawTileMap;

impl Plugin for UIDrawTileMap {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_ui).add_systems(
            Update,
            update_tile_map_color_ui.run_if(
                resource_exists::<TileInfos>.and(
                    resource_changed::<TileInfos>
                        .or(|q: Query<&Grid<TileId>, Changed<Grid<TileId>>>| !q.is_empty()),
                ),
            ),
        );
    }
}

#[derive(Resource)]
struct BodyEntity(Entity);

#[derive(Component)]
struct DisplayMapUI;

fn spawn_debug_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let body = spawn_window(
        &mut commands,
        &asset_server,
        "[Debug] Display Map",
        (DisplayMapUI,),
    )
    .id();

    commands.insert_resource(BodyEntity(body));
}

fn update_tile_map_color_ui(
    body: Res<BodyEntity>,
    q_tiles: Query<&Grid<TileId>>,
    tile_info: Res<TileInfos>,
    mut images: ResMut<Assets<Image>>,
    ui_entity: Local<Option<Entity>>,
    mut commands: Commands,
) {
    let Ok(grid) = q_tiles.single() else {
        return;
    };

    debug!("Updating map");

    let image = images.add(draw_tile_map_colors(grid, tile_info));

    let BodyEntity(body) = *body;
    commands.entity(body).despawn_children().with_child((
        Name::new("Image Container"),
        Node {
            width: percent(100.0),
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        children![ImageNode {
            image,
            flip_y: true,
            ..Default::default()
        }],
    ));
}

fn draw_tile_map_colors(grid: &Grid<TileId>, tile_info_list: Res<TileInfos>) -> Image {
    let data = grid
        .iter()
        .map(|&id| tile_info_list.get(id))
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
                width: grid::WIDTH as u32,
                height: grid::HEIGHT as u32,
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
