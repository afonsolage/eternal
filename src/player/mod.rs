use bevy::{app::Plugin, ecs::component::Component, transform::components::Transform};

use crate::player::controller::PlayerControllerPlugin;

mod controller;

pub use controller::PlayerController;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(PlayerControllerPlugin);
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct Player;
