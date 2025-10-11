use bevy::prelude::*;

use crate::player::controller::PlayerControllerPlugin;

mod controller;

pub use controller::PlayerController;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(PlayerControllerPlugin)
            .add_observer(on_add_player);
    }
}

#[derive(Component)]
pub struct PlayerCamera;

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
