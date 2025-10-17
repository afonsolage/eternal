use avian2d::prelude::LinearVelocity;
use bevy::{
    app::Plugin,
    ecs::component::Component,
    input::{ButtonInput, keyboard::KeyCode},
    prelude::*,
};

use crate::player::{Player, PlayerCamera};

const MAX_LOOKING_AT_DISTANCE: f32 = 1000.0;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_player,
                update_looking_at,
                update_looking_at_pointer,
                trigger_action,
            ),
        )
        .add_observer(on_add_player);
    }
}

#[derive(Component, Reflect)]
#[component(immutable)]
pub struct PlayerLookingAt {
    pub distance: f32,
    pub dir: Vec2,
    pub angle: f32,
}

#[derive(Component)]
struct LookingAtPointer;

#[derive(Event)]
pub struct PlayerAction;

impl Default for PlayerLookingAt {
    fn default() -> Self {
        Self {
            dir: Vec2::X,
            angle: 0.0,
            distance: 0.0,
        }
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

    let (entity, player_transform, looking_at) = player_singleton.into_inner();

    let offset = cursor_world_pos - player_transform.translation().xy();
    let dir = offset.normalize();
    let distance = offset.length().clamp(0.0, MAX_LOOKING_AT_DISTANCE);

    // Avoid updating with micro difference
    if dir.distance(looking_at.dir).abs() > 0.01 {
        // Get the corresponding angle
        let angle = dir.y.atan2(dir.x);
        commands.entity(entity).insert(PlayerLookingAt {
            distance,
            dir,
            angle,
        });
    }
}

fn update_looking_at_pointer(
    looking_at: Single<&PlayerLookingAt, Changed<PlayerLookingAt>>,
    mut pointer: Single<&mut Transform, With<LookingAtPointer>>,
) {
    pointer.rotation = Quat::from_rotation_z(looking_at.angle);

    // offset the pointer on the given direction
    let distance = EasingCurve::new(5.0, 100.0, EaseFunction::SmoothStep)
        .sample(looking_at.distance / MAX_LOOKING_AT_DISTANCE)
        .unwrap_or_default();

    pointer.translation = (looking_at.dir * distance).extend(-0.01);
}

fn trigger_action(
    input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut cooldown: Local<f32>,
    time: Res<Time>,
) {
    if *cooldown > 0.0 {
        *cooldown -= time.delta_secs();
        return;
    }

    if input.pressed(MouseButton::Left) {
        *cooldown = 0.5;
        commands.trigger(PlayerAction);
    }
}
