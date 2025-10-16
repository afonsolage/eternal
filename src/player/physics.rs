use avian2d::prelude::*;
use bevy::prelude::*;

use crate::player::{Player, controller::PlayerLookingAt};

pub struct PlayerPhysicsPlugin;

impl Plugin for PlayerPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_player)
            .add_systems(Update, update_looking_at_ray);
    }
}

pub fn on_add_player(add: On<Add, Player>, mut commands: Commands) {
    commands.entity(add.entity).insert((
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Collider::capsule(8.0, 10.0),
        RayCaster::default(),
        // Disable to avoid annoying flickering when debugging
        SleepingDisabled,
    ));
}

fn update_looking_at_ray(
    singleton: Single<(Entity, &PlayerLookingAt), Changed<PlayerLookingAt>>,
    mut commands: Commands,
) {
    let (entity, looking_at) = singleton.into_inner();

    commands.entity(entity).insert(RayCaster::new(
        Vec2::ZERO,
        Dir2::new_unchecked(looking_at.dir),
    ));
}
