use bevy::{
    image::ToExtents,
    prelude::*,
    render::render_resource::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
    window::PresentMode,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use eternal_procgen::atlas::{self, Atlas};

#[derive(Component)]
struct AtlasImage;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, update_atlas_image.run_if(resource_changed::<Atlas>))
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);

    let image = Image {
        data: Some(vec![
            u8::MAX;
            atlas::DIMS.as_usizevec2().element_product() * 4
        ]), // 4 colors (rgba)
        texture_descriptor: TextureDescriptor {
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            size: atlas::DIMS.as_uvec2().to_extents(),
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };

    commands.spawn((
        Name::new("Root"),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            AtlasImage,
            ImageNode {
                image: images.add(image),
                ..default()
            }
        )],
    ));

    commands.insert_resource(eternal_procgen::generate_atlas());
}

fn update_atlas_image(
    node: Single<&ImageNode, With<AtlasImage>>,
    mut images: ResMut<Assets<Image>>,
    atlas: Res<Atlas>,
) {
    let Some(image) = images.get_mut(node.image.id()) else {
        return;
    };

    let data = image.data.as_mut().unwrap();

    for i in 0..atlas::DIMS.as_usizevec2().element_product() {
        let index = i * 4;
        let elevation = atlas.elevation[i];

        let gray_scale = ((elevation + 1.0) / 2.0 * 255.0) as u8;
        data[index] = gray_scale;
        data[index + 1] = gray_scale;
        data[index + 2] = gray_scale;
    }
}
