use bevy::prelude::*;

use crate::player::{
    camera::PlayerCameraPlugin, controller::PlayerControllerPlugin, physics::PlayerPhysicsPlugin,
};

mod camera;
mod controller;
mod physics;

pub use camera::PlayerCamera;
pub use controller::PlayerController;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            PlayerControllerPlugin,
            PlayerCameraPlugin,
            PlayerPhysicsPlugin,
        ))
        .add_observer(on_add_player);
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct Player;

fn on_add_player(add: On<Add, Player>, mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.entity(add.entity).insert((
        PlayerController::default(),
        Sprite {
            image: asset_server.load("sheets/player.png"),
            ..Default::default()
        },
    ));
}
