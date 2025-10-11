use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    player::Player,
    world::grid::{GridId, GridIdChanged, LayerIndex},
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            PhysicsPickingPlugin,
            PhysicsDebugPlugin,
        ))
        .insert_resource(Gravity::ZERO)
        .insert_gizmo_config(
            PhysicsGizmos::default(),
            GizmoConfig {
                enabled: false,
                ..default()
            },
        )
        .add_observer(on_add_grid)
        .add_observer(on_grid_id_changed);
    }
}

pub fn on_add_player(add: On<Add, Player>, mut commands: Commands) {
    commands.entity(add.entity).insert((
        RigidBody::Dynamic,
        Collider::capsule(8.0, 10.0),
        LockedAxes::ROTATION_LOCKED,
    ));
}

pub fn on_add_grid(add: On<Add, GridId>, mut commands: Commands) {
    commands.entity(add.entity).insert(RigidBody::Static);
}

pub fn on_grid_id_changed(
    _: On<GridIdChanged>,
    singleton: Single<(Entity, &GridId)>,
    mut commands: Commands,
) {
    let (entity, grid) = *singleton;

    let walls = grid[LayerIndex::WALL]
        .positions()
        .filter_map(|(x, y, &id)| {
            if id.is_none() {
                None
            } else {
                Some(IVec2::new(x as i32, y as i32))
            }
        })
        .collect::<Vec<_>>();

    if walls.is_empty() {
        commands.entity(entity).remove::<Collider>();
    } else {
        let wall_collider = Collider::voxels(Vec2::new(32.0, 32.0), &walls);
        commands.entity(entity).insert(wall_collider);
    }
}
