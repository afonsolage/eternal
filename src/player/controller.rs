use avian2d::prelude::LinearVelocity;
use bevy::{
    app::Plugin,
    ecs::component::Component,
    input::{ButtonInput, keyboard::KeyCode},
    prelude::*,
};

use crate::player::{Player, PlayerCamera};

const POINTER_OFFSET: f32 = 20.0;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_player, update_looking_at, update_looking_at_pointer),
        )
        .add_observer(on_add_player);
    }
}

#[derive(Component, Reflect, Deref)]
#[component(immutable)]
pub struct PlayerLookingAt(Vec2);

#[derive(Component)]
struct LookingAtPointer;

impl Default for PlayerLookingAt {
    fn default() -> Self {
        Self(Vec2::X)
    }
}

#[derive(Component, Reflect)]
pub struct PlayerController {
    pub move_speed: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self { move_speed: 100.0 }
    }
}

fn on_add_player(add: On<Add, Player>, mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .entity(add.entity)
        .insert(PlayerLookingAt::default());

    commands.entity(add.entity).with_child((
        LookingAtPointer,
        Transform::default(),
        Sprite {
            image: asset_server.load("sheets/looking_at.png"),
            ..default()
        },
    ));
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

fn update_looking_at(
    window: Single<&Window>,
    cam_singleton: Single<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    player_singleton: Single<(Entity, &GlobalTransform, &PlayerLookingAt)>,
    mut commands: Commands,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (cam, camera_transform) = cam_singleton.into_inner();

    if !cam.is_active {
        return;
    }

    let Ok(cursor_world_pos) = cam.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let (entity, player_transform, &PlayerLookingAt(looking_at)) = player_singleton.into_inner();

    let dir = (cursor_world_pos - player_transform.translation().xy()).normalize();

    // Avoid updating with micro difference
    if dir.distance(looking_at).abs() > 0.01 {
        commands.entity(entity).insert(PlayerLookingAt(dir));
    }
}

fn update_looking_at_pointer(
    looking_at: Single<&PlayerLookingAt, Changed<PlayerLookingAt>>,
    mut pointer: Single<&mut Transform, With<LookingAtPointer>>,
) {
    // Get the corresponding angle
    let angle = looking_at.y.atan2(looking_at.x);

    // Rotate to match the "up" version of the sprite, since the sprite is lookin up by default
    let rotation = angle - std::f32::consts::PI / 2.0;

    pointer.rotation = Quat::from_rotation_z(rotation);

    // offset the pointer on the given direction
    pointer.translation = looking_at.extend(0.09) * POINTER_OFFSET;
}
