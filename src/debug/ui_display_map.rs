#![allow(unused)]
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
    config::tile::TileInfoList,
    ui::window::spawn_window,
    world::{
        map::{self, Map},
        renderer::tilemap::Tilemap,
    },
};

pub struct UIDisplayMap;

impl Plugin for UIDisplayMap {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_ui).add_systems(
            Update,
            display_map.run_if(resource_exists::<TileInfoList>.and(
                resource_changed::<TileInfoList>.or(|q: Query<(), Changed<Tilemap>>| !q.is_empty()),
            )),
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

fn display_map(
    body: Res<BodyEntity>,
    q_map: Query<&Tilemap>,
    mut images: ResMut<Assets<Image>>,
    ui_entity: Local<Option<Entity>>,
    mut commands: Commands,
    tile_info_list: Res<TileInfoList>,
) {
    let Ok(tilemap) = q_map.single() else { return };

    debug!("Updating map");

    let image = images.add(create_map_image(&tilemap.map, &tile_info_list));

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

fn create_map_image(map: &Map, tile_info_list: &TileInfoList) -> Image {
    let data = map
        .types
        .data
        .iter()
        .flat_map(|tt| {
            let info = &tile_info_list.0[tt.0 as usize];
            info.map_color
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
                width: map::WIDTH as u32,
                height: map::HEIGHT as u32,
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
