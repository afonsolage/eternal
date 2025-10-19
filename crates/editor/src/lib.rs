use bevy::{
    color::palettes::css::{DARK_BLUE, GREEN, LIGHT_BLUE},
    image::ToExtents,
    prelude::*,
    render::render_resource::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use eternal_procgen::{
    atlas::{self, Atlas},
    noise::{NoiseChanged, NoisePlugin, NoiseType, Noises},
};

mod camera;

#[derive(Component)]
struct AtlasImage;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
            .add_plugins((NoisePlugin, camera::CameraPlugin))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_atlas_image.run_if(resource_changed::<Atlas>),
                    update_atlas.run_if(on_message::<NoiseChanged>),
                    draw_gizmos,
                ),
            );
    }
}

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

    commands.insert_resource(Atlas::new());
}

fn update_atlas_image(
    node: Single<&Sprite, With<AtlasImage>>,
    mut images: ResMut<Assets<Image>>,
    atlas: Res<Atlas>,
) {
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

    debug!("{min}, {max}");
}

fn update_atlas(mut reader: MessageReader<NoiseChanged>, noises: Noises, mut commands: Commands) {
    if reader
        .read()
        .any(|NoiseChanged(tp)| matches!(tp, NoiseType::Atlas))
    {
        commands.insert_resource(eternal_procgen::generate_atlas(&noises));
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
        UVec2::splat(atlas::MAP_COUNT as u32),
        Vec2::splat(atlas::MAP_RESOLUTION as f32),
        LinearRgba::BLACK,
    );
}
