use bevy::{
    image::ToExtents,
    prelude::*,
    render::render_resource::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
};
use eternal_grid::{
    ecs::TileRegistry,
    grid::{self, LayerIndex},
    tile::NONE_INFO,
};
use eternal_procgen::{biome::BiomeRegistry, map::Map};

use crate::{
    EditorState,
    map_editor::ui::{MapOptions, MapUiPlugin},
};

mod ui;

pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MapUiPlugin)
            .add_systems(OnEnter(EditorState::Map), setup)
            .add_systems(OnExit(EditorState::Map), cleanup)
            .add_systems(
                Update,
                (
                    update_map_image.run_if(
                        resource_exists::<Map>
                            .and(resource_changed::<Map>.or(resource_changed::<MapOptions>)),
                    ),
                    update_map.run_if(resource_changed::<BiomeRegistry>),
                    draw_gizmos,
                )
                    .run_if(in_state(EditorState::Map)),
            );
    }
}

#[derive(Component)]
struct MapImage;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    biome_registry: Res<BiomeRegistry>,
) {
    let image = Image {
        data: Some(vec![0; grid::LAYER_SIZE * 4]), // 4 colors (rgba)
        texture_descriptor: TextureDescriptor {
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            size: grid::DIMS.to_extents(),
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };

    commands.spawn((
        Name::new("Root"),
        MapImage,
        Sprite {
            image: images.add(image),
            ..default()
        },
    ));

    let map = if !biome_registry.is_empty() {
        let biome = biome_registry
            .get_biome("Forest")
            .expect("Biome forest exists");
        eternal_procgen::generate_map(biome)
    } else {
        Map::default()
    };

    commands.insert_resource(map);
}

fn cleanup(mut commands: Commands, single: Option<Single<Entity, With<MapImage>>>) {
    commands.remove_resource::<Map>();
    if let Some(single) = single {
        commands.entity(single.into_inner()).despawn();
    }
}

fn update_map_image(
    node: Single<&Sprite, With<MapImage>>,
    mut images: ResMut<Assets<Image>>,
    registry: Res<TileRegistry>,
    map: Res<Map>,
    opts: Res<MapOptions>,
) {
    let Some(image) = images.get_mut(node.image.id()) else {
        return;
    };

    let data = image.data.as_mut().unwrap();
    data.fill(0);

    let colors: &mut [[u8; 4]] = bytemuck::cast_slice_mut(image.data.as_mut().unwrap());

    let mut min = f32::MAX;
    let mut max = f32::MIN;

    for (i, color) in colors.iter_mut().enumerate() {
        let elevation = *map.elevation[i];

        if elevation < min {
            min = elevation;
        }
        if elevation > max {
            max = elevation;
        }

        if opts.terrain {
            let tile_id = map.tile[LayerIndex::Floor][i];

            let tile_info = registry.get(&tile_id).unwrap_or(&NONE_INFO);
            *color = tile_info.map_color.to_u8_array();
        }

        if opts.flora {
            let tile_id = map.tile[LayerIndex::Wall][i];

            if !tile_id.is_none() {
                let tile_info = registry.get(&tile_id).unwrap_or(&NONE_INFO);
                *color = tile_info.map_color.to_u8_array();
            }
        }
    }

    debug!("{min}, {max}");
}

fn update_map(biome_registry: Res<BiomeRegistry>, mut commands: Commands) {
    let biome = biome_registry
        .get_biome("Forest")
        .expect("Biome forest exists");
    commands.insert_resource(eternal_procgen::generate_map(biome));
}

fn draw_gizmos(mut gizmos: Gizmos, projetion: Single<&Projection, With<Camera2d>>) {
    let Projection::Orthographic(orto) = projetion.into_inner() else {
        return;
    };

    if orto.scale > 0.2 {
        return;
    }

    gizmos.grid_2d(
        Isometry2d::IDENTITY,
        UVec2::splat(256),
        Vec2::splat(1.0),
        LinearRgba::BLACK,
    );
}
