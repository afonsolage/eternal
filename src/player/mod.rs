use bevy::prelude::*;

use crate::player::{
    actions::PlayerActionsPlugin, camera::PlayerCameraPlugin, controller::PlayerControllerPlugin,
    physics::PlayerPhysicsPlugin,
};

mod actions;
pub use actions::PlayerActionHit;

mod camera;
pub use camera::PlayerCamera;

mod controller;
pub use controller::PlayerController;

mod physics;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            PlayerControllerPlugin,
            PlayerCameraPlugin,
            PlayerPhysicsPlugin,
            PlayerActionsPlugin,
        ))
        .add_observer(on_add_player);
    }
}

#[derive(Component)]
#[require(Transform, Sprite)]
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
