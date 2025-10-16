use avian2d::{
    parry::query::RayCast,
    prelude::{RayCaster, RayHits},
};
use bevy::{prelude::*, ui_widgets::observe};

use crate::{
    effects::{FxSwipe, FxSwipeHit},
    player::{
        Player,
        controller::{PlayerAction, PlayerLookingAt},
    },
};
pub struct PlayerActionsPlugin;

#[derive(EntityEvent, Debug)]
pub struct PlayerActionHit {
    #[event_target]
    pub entity: Entity,
    #[allow(unused)]
    pub hit_source: Entity,
    pub collision_source: Entity,
}

impl Plugin for PlayerActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_player_action);
    }
}

fn on_player_action(
    _: On<PlayerAction>,
    mut commands: Commands,
    singleton: Single<(Entity, &PlayerLookingAt, &RayHits), With<Player>>,
) {
    let (entity, looking_at, ray_hits) = singleton.into_inner();

    let offset = ray_hits
        .iter_sorted()
        .next()
        .map(|hit| hit.distance)
        .unwrap_or(20.0)
        .min(20.0);

    commands.entity(entity).with_child((
        FxSwipe,
        Transform::from_translation((looking_at.dir * offset).extend(0.1))
            .with_rotation(Quat::from_rotation_z(looking_at.angle))
            .with_scale(Vec3::splat(2.0)),
        observe(on_player_action_hit),
    ));
}

fn on_player_action_hit(hit: On<FxSwipeHit>, mut commands: Commands) {
    commands.trigger(PlayerActionHit {
        entity: hit.target,
        hit_source: hit.entity,
        collision_source: hit.source,
    });
}
