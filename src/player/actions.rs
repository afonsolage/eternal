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
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
    looking_at: Single<&PlayerLookingAt>,
) {
    commands.entity(*player).with_child((
        FxSwipe,
        Transform::from_translation((looking_at.dir * 20.0).extend(0.1))
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
