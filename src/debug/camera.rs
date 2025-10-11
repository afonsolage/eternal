use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use crate::player::PlayerCamera;

pub struct DebugCameraPlugin;

impl Plugin for DebugCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, ((zoom, pan).run_if(camera_enabled), toggle_camera));
    }
}

#[derive(Component)]
struct DebugCamera {
    zoom_speed: f32,
    pan_speed: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug Camera"),
        DebugCamera {
            zoom_speed: 0.2,
            pan_speed: 0.8,
        },
        Camera2d,
        Camera {
            is_active: false,
            ..default()
        },
    ));
}

fn camera_enabled(camera: Single<&Camera, With<DebugCamera>>) -> bool {
    camera.is_active
}

#[allow(clippy::type_complexity)]
fn toggle_camera(
    input: Res<ButtonInput<KeyCode>>,
    mut debug_singleton: Single<(Entity, &mut Camera), (With<DebugCamera>, Without<PlayerCamera>)>,
    mut player_singleton: Single<(Entity, &mut Camera), (With<PlayerCamera>, Without<DebugCamera>)>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::KeyP) {
        let (debug_entity, ref mut debug_cam) = *debug_singleton;
        let (player_entity, ref mut player_cam) = *player_singleton;

        let is_active = !debug_cam.is_active;

        debug_cam.is_active = is_active;
        player_cam.is_active = !is_active;

        if debug_cam.is_active {
            commands.entity(debug_entity).insert(IsDefaultUiCamera);
            commands.entity(player_entity).remove::<IsDefaultUiCamera>();
        } else {
            commands.entity(debug_entity).remove::<IsDefaultUiCamera>();
            commands.entity(player_entity).insert(IsDefaultUiCamera);
        }
    }
}

fn zoom(
    mut scroll_msgs: MessageReader<MouseWheel>,
    mut singleton: Single<(&mut Projection, &DebugCamera)>,
) {
    let (ref mut projection, camera) = *singleton;

    let Projection::Orthographic(ortho) = projection.as_mut() else {
        return;
    };

    for ev in scroll_msgs.read() {
        let zoom_delta = ev.y * camera.zoom_speed;
        ortho.scale -= zoom_delta;
        ortho.scale = ortho.scale.clamp(0.1, 10.0);
    }
}

fn pan(
    mut motion_msgs: MessageReader<MouseMotion>,
    input: Res<ButtonInput<MouseButton>>,
    mut singleton: Single<(&mut Transform, &Projection, &DebugCamera)>,
) {
    if !input.pressed(MouseButton::Middle) {
        return;
    }

    for ev in motion_msgs.read() {
        let (ref mut transform, projection, camera) = *singleton;
        let scale = if let Projection::Orthographic(ortho) = projection {
            ortho.scale
        } else {
            1.0
        };

        transform.translation.x -= ev.delta.x * camera.pan_speed * scale;
        transform.translation.y += ev.delta.y * camera.pan_speed * scale;
    }
}
