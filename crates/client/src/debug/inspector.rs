use bevy::{camera::visibility::RenderLayers, prelude::*};
use bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
            .insert_resource(EguiGlobalSettings {
                auto_create_primary_context: false,
                ..default()
            })
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    debug!("Spawning inspector cam!");

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
