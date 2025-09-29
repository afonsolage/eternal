use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

use crate::world::{
    grid::{self, Grid},
    renderer::tilemap::Tilemap,
    tile::{self, TileId, TileRegistry},
};

pub struct DrawGridsPlugin;

impl Plugin for DrawGridsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
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

fn draw_grid_texts(camera: Query<&Camera>) {
    let Ok(camera) = camera.single() else {
        return;
    };
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
