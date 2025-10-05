use bevy::{
    asset::RenderAssetUsages,
    feathers::controls::checkbox,
    math::U16Vec2,
    mesh::PrimitiveTopology,
    prelude::*,
    ui::Checked,
    ui_widgets::{ValueChange, observe},
};

use crate::{
    ui::window::{WindowConfig, window},
    world::{
        grid::{self, Grid, GridElevation, GridId, GridVisible, LayerIndex},
        renderer::tilemap::Tilemap,
        tile::{self, TileRegistry, TileVisible},
    },
};

const WIREFRAME_HEIGHT: f32 = 0.3;
const INFO_HEIGHT: f32 = 0.2;
const IDS_HEIGHT: f32 = 0.2;

pub struct DrawGridsPlugin;

impl Plugin for DrawGridsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_grids_ui);
        app.add_systems(
            Update,
            (
                draw_grid_info.run_if(
                    resource_changed::<DrawGridsConfig>
                        .or(|q: Query<(), Changed<Grid<TileVisible>>>| !q.is_empty()),
                ),
                draw_grid_wireframe.run_if(
                    resource_exists::<TileRegistry>.and(
                        resource_changed::<TileRegistry>.or(resource_changed::<DrawGridsConfig>),
                    ),
                ),
                draw_grid_tile_ids.run_if(
                    resource_exists::<TileRegistry>.and(
                        resource_changed::<TileRegistry>
                            .or(resource_changed::<DrawGridsConfig>)
                            .or(|q: Query<(), Changed<Tilemap>>| !q.is_empty()),
                    ),
                ),
            ),
        )
        .add_observer(on_add_tilemap_insert_cache)
        .init_resource::<DrawGridsConfig>();
    }
}

#[derive(Default, Copy, Clone, Resource)]
struct DrawGridsConfig {
    show_ids: bool,
    show_grid: bool,
    show_info: bool,
}

#[derive(Component)]
struct DrawGridsUi;

fn spawn_debug_grids_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug Grids"),
        window(
            WindowConfig {
                title: "[Debug] Draw Grids".to_string(),
                right: px(1.0),
                bottom: px(1.0),
                ..default()
            },
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    (
                        checkbox((), Spawn(Text::new("Show IDs"))),
                        observe(
                            |change: On<ValueChange<bool>>,
                             mut commands: Commands,
                             mut config: ResMut<DrawGridsConfig>| {
                                config.show_ids = change.value;
                                if config.show_ids {
                                    commands.entity(change.source).insert(Checked);
                                } else {
                                    commands.entity(change.source).remove::<Checked>();
                                }
                            }
                        ),
                    ),
                    (
                        checkbox((), Spawn(Text::new("Show Grid"))),
                        observe(
                            |change: On<ValueChange<bool>>,
                             mut commands: Commands,
                             mut config: ResMut<DrawGridsConfig>| {
                                config.show_grid = change.value;
                                if config.show_grid {
                                    commands.entity(change.source).insert(Checked);
                                } else {
                                    commands.entity(change.source).remove::<Checked>();
                                }
                            }
                        ),
                    ),
                    (
                        checkbox((), Spawn(Text::new("Show Info"))),
                        observe(
                            |change: On<ValueChange<bool>>,
                             mut commands: Commands,
                             mut config: ResMut<DrawGridsConfig>| {
                                config.show_info = change.value;
                                if config.show_info {
                                    commands.entity(change.source).insert(Checked);
                                } else {
                                    commands.entity(change.source).remove::<Checked>();
                                }
                            }
                        ),
                    )
                ],
            ),
        ),
        DrawGridsUi,
    ));
}

fn format_tile_info(index: usize, ids: &GridId, elevations: &GridElevation) -> String {
    let layer = LayerIndex::FLOOR;
    format!(
        "{layer:?}: {}\nele: {:.02}",
        *ids[layer][index], *elevations[index]
    )
}

fn tile_info_bundle(tile_size: Vec2, index: usize, info: String) -> impl Bundle {
    let tile_pos = U16Vec2::new(
        index as u16 % grid::DIMS.x as u16,
        index as u16 / grid::DIMS.x as u16,
    );
    let tile_center = (tile_pos.as_vec2() * tile_size + (tile_size / 2.0)).extend(INFO_HEIGHT);

    (
        Name::new(format!("Tile Info {}, {}", tile_pos.x, tile_pos.y)),
        Text2d::default(),
        Transform::from_translation(tile_center).with_scale(Vec3::splat(0.5)),
        children![
            (
                Transform::from_xyz(0.0, 23.0, 0.0),
                Text2d::default(),
                children![(
                    Text2d::new(format!("{},{}", tile_pos.x, tile_pos.y)),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    TextFont {
                        font_size: 12.0,
                        ..Default::default()
                    }
                )],
            ),
            (
                Transform::from_xyz(0.0, 0.0, 0.0),
                Text2d::new(info),
                TextFont {
                    font_size: 10.0,
                    ..Default::default()
                }
            )
        ],
    )
    //
}

fn on_add_tilemap_insert_cache(add: On<Add, Tilemap>, mut commands: Commands) {
    let root = commands
        .spawn((
            Name::new("Grid Overlay - info"),
            Transform::default(),
            Text2d::default(),
        ))
        .id();

    commands
        .entity(add.entity)
        .insert(DrawGridInfoCache {
            root,
            entities: Grid::new(),
        })
        .observe(
            |remove: On<Remove, DrawGridInfoCache>,
             mut commands: Commands,
             q: Query<&DrawGridInfoCache>| {
                if let Ok(cache) = q.get(remove.entity) {
                    commands.entity(cache.root).despawn();
                }
            },
        );
}

#[derive(Component)]
struct DrawGridInfoCache {
    root: Entity,
    entities: Grid<Option<Entity>>,
}

#[allow(clippy::type_complexity)]
fn draw_grid_info(
    q_tilemaps: Query<(
        &Tilemap,
        &GridVisible,
        &GridId,
        &GridElevation,
        &mut DrawGridInfoCache,
    )>,
    config: Res<DrawGridsConfig>,
    mut commands: Commands,
) {
    for (tilemap, grid_visible, grid_id, grid_elevation, mut cache) in q_tilemaps {
        debug!("Updating grid tile infos!");

        // Despawn all text entities if config says so
        if !config.show_info {
            cache
                .entities
                .iter_mut()
                .filter_map(Option::take)
                .for_each(|e| {
                    commands.entity(e).despawn();
                });
            return;
        }

        // Avoid spawning a huge number of infos when the camera zooms out
        if grid_visible.iter().filter(|t| t.is_visible()).count() > 512 {
            cache.entities.iter_mut().for_each(|t| {
                if let Some(e) = t.take() {
                    commands.entity(e).despawn();
                }
            });
            return;
        }

        let mut despawn = vec![];
        commands.entity(cache.root).with_children(|parent| {
            grid_visible
                .iter()
                .enumerate()
                .for_each(|(index, tile_visible)| {
                    if !tile_visible.is_visible()
                        && let Some(entity) = cache.entities[index]
                    {
                        despawn.push(entity);
                        cache.entities[index] = None;
                    } else if tile_visible.is_visible() && cache.entities[index].is_none() {
                        let info = format_tile_info(index, grid_id, grid_elevation);
                        let entity = parent
                            .spawn(tile_info_bundle(tilemap.tile_size, index, info))
                            .id();
                        cache.entities[index] = Some(entity);
                    }
                });
        });

        despawn
            .into_iter()
            .for_each(|e| commands.entity(e).despawn());
    }
}

fn draw_grid_wireframe(
    q_tiles: Query<(&GridId, &Tilemap)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut entities: Local<Vec<Entity>>,
    config: Res<DrawGridsConfig>,
) {
    entities
        .drain(..)
        .for_each(|e| commands.entity(e).despawn());

    if !config.show_grid {
        return;
    }

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
                Transform::from_xyz(0.0, 0.0, WIREFRAME_HEIGHT)
                    .with_scale(tilemap.tile_size.extend(1.0)),
            ))
            .id();

        entities.push(entity);
    }
}

fn draw_grid_tile_ids(
    q_tiles: Query<(&GridId, &Tilemap)>,
    registry: Res<TileRegistry>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut entities: Local<Vec<Entity>>,
    config: Res<DrawGridsConfig>,
) {
    entities
        .drain(..)
        .for_each(|e| commands.entity(e).despawn());

    if !config.show_ids {
        return;
    }

    debug!("Drawing grid tile ids");

    for (grid, tilemap) in q_tiles {
        let layer = &grid[LayerIndex::FLOOR];

        let mut positions = vec![];
        let mut colors = vec![];
        let mut indices = vec![];

        let mut i = 0;
        for y in 0..grid::DIMS.y {
            for x in 0..grid::DIMS.x {
                let info = registry
                    .get(layer.get(x as u16, y as u16))
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
                Transform::from_xyz(0.0, 0.0, IDS_HEIGHT).with_scale(tilemap.tile_size.extend(1.0)),
            ))
            .id();

        entities.push(entity);
    }
}
