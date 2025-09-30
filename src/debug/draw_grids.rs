use bevy::{
    asset::RenderAssetUsages, ecs::entity::EntityHashMap, math::U16Vec2, mesh::PrimitiveTopology,
    prelude::*,
};

use crate::world::{
    grid::{self, Grid},
    renderer::tilemap::Tilemap,
    tile::{self, TileId, TileRegistry, TileVisible},
};

pub struct DrawGridsPlugin;

impl Plugin for DrawGridsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_grid_texts,
                draw_grid_wireframe.run_if(resource_exists_and_changed::<TileRegistry>),
                draw_grid_tile_ids.run_if(
                    resource_exists::<TileRegistry>.and(
                        resource_changed::<TileRegistry>
                            .or(|q: Query<(), Changed<Tilemap>>| !q.is_empty()),
                    ),
                ),
            ),
        );
    }
}

#[allow(clippy::type_complexity)]
fn draw_grid_texts(
    q_tilemaps: Query<
        (Entity, &Tilemap, &Grid<TileVisible>, &Grid<TileId>),
        Changed<Grid<TileVisible>>,
    >,
    mut entity_cache: Local<EntityHashMap<Vec<Option<Entity>>>>,
    mut commands: Commands,
) {
    for (entity, tilemap, grid_visible, grid_id) in q_tilemaps {
        debug!("Updating grid texts!");

        let entities =
            entity_cache
                .entry(entity)
                .or_insert(vec![None; grid::DIMS.element_product() as usize]);

        grid_visible
            .iter()
            .enumerate()
            .for_each(|(index, tile_visible)| {
                if !tile_visible.is_visible()
                    && let Some(entity) = entities[index]
                {
                    commands.entity(entity).despawn();
                    entities[index] = None;
                } else if tile_visible.is_visible() && entities[index].is_none() {
                    let tile_pos =
                        UVec2::new(index as u32 % grid::DIMS.x, index as u32 / grid::DIMS.x);
                    let tile_world_pos =
                        tile_pos.as_vec2() * tilemap.tile_size + (tilemap.tile_size / 2.0);

                    let entity = commands
                        .spawn((
                            Name::new(format!("Tile Text {}, {}", tile_pos.x, tile_pos.y)),
                            Text2d::new(grid_id[index].to_string()),
                            Transform::from_translation(tile_world_pos.extend(102.0))
                                .with_scale(Vec3::splat(0.5)),
                        ))
                        .id();

                    entities[index] = Some(entity);
                }
            });
    }
}

fn draw_grid_wireframe(
    q_tiles: Query<(&Grid<TileId>, &Tilemap)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut entities: Local<Vec<Entity>>,
) {
    entities
        .drain(..)
        .for_each(|e| commands.entity(e).despawn());

    debug!("Drawing grid wireframe");

    for (_grid, tilemap) in q_tiles {
        let mut positions = vec![];

        for x in 0..grid::DIMS.x {
            let (x, y) = (x as f32, grid::DIMS.y as f32);
            positions.push([x, 0.0, 0.0]);
            positions.push([x, y, 0.0]);
        }
        for y in 0..grid::DIMS.y {
            let (x, y) = (grid::DIMS.x as f32, y as f32);
            positions.push([0.0, y, 0.0]);
            positions.push([x, y, 0.0]);
        }

        let mesh = meshes.add(
            Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions),
        );

        let material = materials.add(ColorMaterial {
            color: Color::BLACK,
            ..Default::default()
        });

        let entity = commands
            .spawn((
                Name::new("Grid Overlay - wireframe"),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 110.0).with_scale(tilemap.tile_size.extend(1.0)),
            ))
            .id();

        entities.push(entity);
    }
}

fn draw_grid_tile_ids(
    q_tiles: Query<(&Grid<TileId>, &Tilemap)>,
    registry: Res<TileRegistry>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut entities: Local<Vec<Entity>>,
) {
    entities
        .drain(..)
        .for_each(|e| commands.entity(e).despawn());

    debug!("Drawing grid tile ids");

    for (grid, tilemap) in q_tiles {
        let mut positions = vec![];
        let mut colors = vec![];
        let mut indices = vec![];

        let mut i = 0;
        for y in 0..grid::DIMS.y {
            for x in 0..grid::DIMS.x {
                let info = registry
                    .get(grid.get(x as u16, y as u16))
                    .unwrap_or(&tile::NONE_INFO);

                let (x, y) = (x as f32, y as f32);
                positions.extend([
                    [x, y, 0.0],
                    [x + 1.0, y, 0.0],
                    [x + 1.0, y + 1.0, 0.0],
                    [x, y + 1.0, 0.0],
                ]);

                indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
                i += 4;

                let color = info.map_color.with_alpha(0.25);
                colors.extend(vec![color.to_f32_array(); 4]);
            }
        }

        let mesh = meshes.add(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
            .with_inserted_indices(bevy::mesh::Indices::U32(indices)),
        );

        let material = materials.add(ColorMaterial {
            ..Default::default()
        });

        let entity = commands
            .spawn((
                Name::new("Grid Overlay - TileIds"),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 100.0).with_scale(tilemap.tile_size.extend(1.0)),
            ))
            .id();

        entities.push(entity);
    }
}
