use std::ops::Deref;

use avian2d::prelude::LinearVelocity;
use bevy::{
    app::Plugin,
    ecs::component::Component,
    input::{ButtonInput, keyboard::KeyCode},
    prelude::*,
};

use crate::player::PlayerCamera;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_player, look_at));
    }
}

#[derive(Component, Reflect)]
pub struct PlayerController {
    pub move_speed: f32,
    pub looking_at: Vec2,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_speed: 100.0,
            looking_at: Vec2::X,
        }
    }
}

fn move_player(
    singleton: Single<(&PlayerController, &mut LinearVelocity)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec2::ZERO;
    let (controller, mut velocity) = singleton.into_inner();

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

fn look_at(
    window: Single<&Window>,
    cam_singleton: Single<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    player_singleton: Single<(&GlobalTransform, &mut PlayerController)>,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (cam, camera_transform) = cam_singleton.into_inner();

    let Ok(cursor_world_pos) = cam.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let (player_transform, controller) = player_singleton.deref();

    let dir = (cursor_world_pos - player_transform.translation().xy()).normalize();

    if dir.distance(controller.looking_at).abs() < 0.01 {
        return;
    }

    // Only borrow mutable when needed
    let (_, mut controller) = player_singleton.into_inner();
    controller.looking_at = dir;
}
