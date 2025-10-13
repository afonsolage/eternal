use bevy::prelude::*;

use crate::{
    effects::FxSwipe,
    player::{
        Player,
        controller::{PlayerAction, PlayerLookingAt},
    },
};
pub struct PlayerActionsPlugin;

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
        Transform::from_translation((looking_at.dir * 20.0).extend(0.2))
            .with_rotation(Quat::from_rotation_z(looking_at.angle))
            .with_scale(Vec3::splat(2.0)),
    ));
}
