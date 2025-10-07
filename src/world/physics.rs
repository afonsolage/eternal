use avian2d::prelude::*;
use bevy::prelude::*;

use crate::world::grid::{GridId, LayerIndex};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // 32 pixels per unit
        app.add_plugins((
            PhysicsPlugins::default().with_length_unit(32.0),
            //PhysicsDebugPlugin,
        ));
    }
}

pub fn generate_collisions(ids: &GridId) -> impl Bundle {
    let walls = ids[LayerIndex::WALL]
        .positions()
        .filter_map(|(x, y, &id)| {
            if id.is_none() {
                None
            } else {
                Some(IVec2::new(x as i32, y as i32))
            }
        })
        .collect::<Vec<_>>();

    let wall_collider = Collider::voxels(Vec2::new(32.0, 32.0), &walls);

    (wall_collider, RigidBody::Static)
}
