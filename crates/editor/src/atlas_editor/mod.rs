use bevy::{
    image::ToExtents,
    prelude::*,
    render::render_resource::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
};
use eternal_procgen::atlas::{self, Atlas};

use crate::EditorState;

pub struct AtlasEditorPlugin;

impl Plugin for AtlasEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(EditorState::Atlas),
            (setup, update_atlas_image.run_if(resource_exists::<Atlas>)).chain(),
        )
        .add_systems(OnExit(EditorState::Atlas), cleanup)
        .add_systems(
            Update,
            (
                update_atlas_image.run_if(resource_exists_and_changed::<Atlas>),
                draw_gizmos,
            )
                .run_if(in_state(EditorState::Atlas)),
        );
    }
}

#[derive(Component)]
struct AtlasImage;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = Image {
        data: Some(vec![u8::MAX; atlas::ATLAS_SIZE * 4]), // 4 colors (rgba)
        texture_descriptor: TextureDescriptor {
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            size: UVec2::splat(atlas::ATLAS_AXIS_SIZE as u32).to_extents(),
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };

    commands.spawn((
        Name::new("Root"),
        AtlasImage,
        Sprite {
            image: images.add(image),
            ..default()
        },
    ));
}

fn cleanup(mut commands: Commands, single: Option<Single<Entity, With<AtlasImage>>>) {
    if let Some(single) = single {
        commands.entity(single.into_inner()).despawn();
    }
}

fn update_atlas_image(
    node: Single<&Sprite, With<AtlasImage>>,
    mut images: ResMut<Assets<Image>>,
    atlas: Res<Atlas>,
) {
    use bevy::color::palettes::css::*;

    debug!("Updating atlas image!");

    let Some(image) = images.get_mut(node.image.id()) else {
        return;
    };

    let data = image.data.as_mut().unwrap();
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    for i in 0..atlas::ATLAS_SIZE {
        let index = i * 4;
        let elevation = atlas.elevation[i];

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

    debug!("min: {min}, max: {max}");
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
        UVec2::splat(atlas::MAP_COUNT as u32),
        Vec2::splat(atlas::MAP_RESOLUTION as f32),
        LinearRgba::BLACK,
    );
}
