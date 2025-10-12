use avian2d::prelude::LinearVelocity;
use bevy::{
    app::Plugin,
    ecs::component::Component,
    input::{ButtonInput, keyboard::KeyCode},
    prelude::*,
};

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player);
    }
}

#[derive(Component)]
pub struct PlayerController {
    pub move_speed: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self { move_speed: 100.0 }
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
