use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    frame_time_graph_config: FrameTimeGraphConfig {
                        min_fps: 120.0,
                        target_fps: 144.0,
                        ..default()
                    },
                    ..default()
                },
            },
        ));
    }
}
