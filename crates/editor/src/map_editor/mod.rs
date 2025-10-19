use bevy::{
    image::ToExtents,
    prelude::*,
    render::render_resource::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
};
use eternal_procgen::{
    map::{self, Map},
    noise::{NoiseChanged, NoiseType, Noises},
};

use crate::EditorState;

pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(EditorState::Map), setup)
            .add_systems(OnExit(EditorState::Map), cleanup)
            .add_systems(
                Update,
                (
                    update_map_image.run_if(resource_exists_and_changed::<Map>),
                    update_map.run_if(on_message::<NoiseChanged>),
                    draw_gizmos,
                )
                    .run_if(in_state(EditorState::Map)),
            );
    }
}

#[derive(Component)]
struct MapImage;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, noises: Noises) {
    let image = Image {
        data: Some(vec![u8::MAX; map::MAP_SIZE * 4]), // 4 colors (rgba)
        texture_descriptor: TextureDescriptor {
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            size: UVec2::splat(map::MAP_AXIS_SIZE as u32).to_extents(),
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

    let map = if noises.is_ready() {
        eternal_procgen::generate_map(&noises)
    } else {
        Map::new()
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
    map: Res<Map>,
) {
    use bevy::color::palettes::css::*;

    let Some(image) = images.get_mut(node.image.id()) else {
        return;
    };

    let data = image.data.as_mut().unwrap();
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    for i in 0..map::MAP_SIZE {
        let index = i * 4;
        let elevation = map.elevation[i];

        if elevation < min {
            min = elevation;
        }
        if elevation > max {
            max = elevation;
        }

        let color = if elevation > 0.25 {
            Srgba::WHITE.to_u8_array()
        } else if elevation > 0.0 {
            GREEN.to_u8_array()
        } else if elevation > -0.25 {
            LIGHT_BLUE.to_u8_array()
        } else {
            DARK_BLUE.to_u8_array()
        };

        data[index] = color[0];
        data[index + 1] = color[1];
        data[index + 2] = color[2];
    }

    debug!("{min}, {max}");
}

fn update_map(mut reader: MessageReader<NoiseChanged>, noises: Noises, mut commands: Commands) {
    if reader
        .read()
        .any(|NoiseChanged(tp)| matches!(tp, NoiseType::Map))
    {
        commands.insert_resource(eternal_procgen::generate_map(&noises));
    }

    reader.clear();
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
