use avian2d::prelude::*;
use bevy::prelude::*;

use crate::player::Player;

pub struct PlayerPhysicsPlugin;

impl Plugin for PlayerPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_player);
    }
}

pub fn on_add_player(add: On<Add, Player>, mut commands: Commands) {
    commands.entity(add.entity).insert((
        RigidBody::Dynamic,
        Collider::capsule(8.0, 10.0),
        LockedAxes::ROTATION_LOCKED,
        // Disable to avoid annoying flickering when debugging
        SleepingDisabled,
    ));
}
