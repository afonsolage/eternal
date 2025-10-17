use bevy::{camera::visibility::RenderLayers, prelude::*};
use bevy_egui::{EguiPlugin, PrimaryEguiContext};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
            .add_systems(PreStartup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Inspector Cam"),
        Camera2d,
        Camera {
            order: 10,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(10),
        PrimaryEguiContext,
    ));
}
