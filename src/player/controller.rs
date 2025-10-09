use avian2d::prelude::LinearVelocity;
use bevy::{
    app::Plugin,
    ecs::{component::Component, system::Query},
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion, MouseWheel},
    },
    prelude::*,
    transform::components::Transform,
};

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_player, zoom_player, pan_camera));
    }
}

#[derive(Component)]
pub struct PlayerController {
    pub move_speed: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_speed: 100.0,
            zoom_speed: 0.2,
            pan_speed: 0.8,
        }
    }
}

fn move_player(
    mut query: Single<(&PlayerController, &mut LinearVelocity)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec2::ZERO;
    let (controller, ref mut velocity) = *query;

    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
    }

    velocity.x = direction.x * controller.move_speed;
    velocity.y = direction.y * controller.move_speed;
}

fn zoom_player(
    mut scroll_evr: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
    player_query: Query<&PlayerController>,
) {
    let Ok(controller) = player_query.single() else {
        return;
    };
    let Ok(mut projection) = camera_query.single_mut() else {
        return;
    };
    let Projection::Orthographic(ortho) = projection.as_mut() else {
        return;
    };

    for ev in scroll_evr.read() {
        let zoom_delta = ev.y * controller.zoom_speed;
        ortho.scale -= zoom_delta;
        ortho.scale = ortho.scale.clamp(0.1, 10.0);
    }
}

fn pan_camera(
    mut motion_evr: MessageReader<MouseMotion>,
    input: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<(&mut Transform, &Projection), With<Camera2d>>,
    player_query: Query<&PlayerController>,
) {
    if !input.pressed(MouseButton::Middle) {
        return;
    }

    let Ok(controller) = player_query.single() else {
        return;
    };

    for ev in motion_evr.read() {
        for (mut transform, projection) in camera_query.iter_mut() {
            let scale = if let Projection::Orthographic(ortho) = projection {
                ortho.scale
            } else {
                1.0
            };

            transform.translation.x -= ev.delta.x * controller.pan_speed * scale;
            transform.translation.y += ev.delta.y * controller.pan_speed * scale;
        }
    }
}
