use avian2d::prelude::*;
use bevy::prelude::*;

use crate::world::grid::{GridId, LayerIndex};

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
        .add_systems(Update, update_wall_collider)
        .add_observer(on_add_grid);
    }
}

pub fn on_add_grid(add: On<Add, GridId>, mut commands: Commands) {
    commands.entity(add.entity).insert(RigidBody::Static);
}

pub fn update_wall_collider(
    singleton: Single<(Entity, &GridId), Changed<GridId>>,
    mut commands: Commands,
) {
    let (entity, grid) = singleton.into_inner();

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
