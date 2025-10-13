use bevy::prelude::*;

use crate::player::{
    actions::PlayerActionsPlugin, camera::PlayerCameraPlugin, controller::PlayerControllerPlugin,
    physics::PlayerPhysicsPlugin, pixel_perfect::PixelPerfectPlugin,
};

mod actions;

mod camera;
pub use camera::PlayerCamera;

mod controller;
pub use controller::PlayerController;

mod physics;

mod pixel_perfect;
pub use pixel_perfect::PixelPerfectCollider;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            PlayerControllerPlugin,
            PlayerCameraPlugin,
            PlayerPhysicsPlugin,
            PlayerActionsPlugin,
            PixelPerfectPlugin,
        ))
        .add_observer(on_add_player);
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct Player;

fn on_add_player(add: On<Add, Player>, mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.entity(add.entity).insert((
        Name::new("Player"),
        PlayerController::default(),
        Sprite {
            image: asset_server.load("sheets/player.png"),
            ..Default::default()
        },
    ));
}
