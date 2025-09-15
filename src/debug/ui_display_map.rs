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
    tilemap::{Tilemap, TilemapChunkMap},
    ui::window::spawn_window,
    world::map::{self, Map},
};

pub struct UIDisplayMap;

impl Plugin for UIDisplayMap {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_ui)
            .add_systems(Update, display_map);
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
        (DisplayMapUI),
    )
    .with_children(|parent| {
        //
    })
    .id();

    commands.insert_resource(BodyEntity(body));
}

fn display_map(
    body: Res<BodyEntity>,
    maybe_map: Option<Res<Map>>,
    mut images: ResMut<Assets<Image>>,
    ui_entity: Local<Option<Entity>>,
    mut commands: Commands,
) {
    let Some(map) = maybe_map else { return };

    if !map.is_changed() {
        return;
    }

    let image = images.add(create_map_image(&map));

    let BodyEntity(body) = *body;
    commands
        .entity(body)
        .despawn_children()
        .with_child(ImageNode {
            image,
            ..Default::default()
        });
}

fn create_map_image(map: &Map) -> Image {
    let data = map
        .types
        .data
        .iter()
        .flat_map(|tt| {
            tt.color()
                .to_linear()
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
