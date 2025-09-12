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
            zoom_speed: 0.1,
            pan_speed: 0.5,
        }
    }
}

fn move_player(
    mut query: Query<(&PlayerController, &mut Transform)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (controller, mut transform) in query.iter_mut() {
        let mut direction = transform.translation;

        if input.pressed(KeyCode::KeyW) {
            direction.y += controller.move_speed * time.delta_secs();
        }
        if input.pressed(KeyCode::KeyS) {
            direction.y -= controller.move_speed * time.delta_secs();
        }
        if input.pressed(KeyCode::KeyA) {
            direction.x -= controller.move_speed * time.delta_secs();
        }
        if input.pressed(KeyCode::KeyD) {
            direction.x += controller.move_speed * time.delta_secs();
        }

        transform.translation = direction;
    }
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
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&PlayerController>,
) {
    if !input.pressed(MouseButton::Middle) {
        return;
    }

    let Ok(controller) = player_query.single() else {
        return;
    };

    for ev in motion_evr.read() {
        for mut transform in camera_query.iter_mut() {
            transform.translation.x -= ev.delta.x * controller.pan_speed;
            transform.translation.y += ev.delta.y * controller.pan_speed;
        }
    }
}
