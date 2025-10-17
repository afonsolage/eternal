use avian2d::prelude::Collisions;
use bevy::prelude::*;

use crate::{
    player::PlayerActionHit,
    world::{
        grid::{GridId, LayerIndex},
        tile::{self, TileId},
    },
};

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_grid);
    }
}

fn on_add_grid(add: On<Add, GridId>, mut commands: Commands) {
    commands.entity(add.entity).observe(on_wall_hit_by_player);
}

fn on_wall_hit_by_player(hit: On<PlayerActionHit>, collisions: Collisions, grid: Single<&GridId>) {
    collisions
        .get(hit.event_target(), hit.collision_source)
        .iter()
        .for_each(|pair| {
            pair.manifolds.iter().for_each(|contact| {
                contact
                    .points
                    .iter()
                    .map(|p| p.point.as_u16vec2() / tile::SIZE)
                    .for_each(|grid_pos| {
                        grid[LayerIndex::Wall].queue(grid_pos.x, grid_pos.y, TileId::default());
                    });
            });
        });
}
