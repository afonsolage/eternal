use std::time::Duration;

use avian2d::prelude::PhysicsGizmos;
use bevy::{
    asset::RenderAssetUsages,
    feathers::controls::{SliderProps, checkbox, slider},
    math::U16Vec2,
    mesh::PrimitiveTopology,
    prelude::*,
    ui::Checked,
    ui_widgets::{SliderPrecision, SliderStep, ValueChange, observe, slider_self_update},
};

use crate::{
    effects::FxFpsMultiplier,
    run_conditions::{component_changed, timeout},
    ui::{
        controls::spacer,
        window::{WindowConfig, window},
    },
    world::renderer::tilemap::{Tilemap, TilemapCache, TilemapChunkMaterial},
};
use eternal_grid::{
    grid::{self, Grid, GridElevation, GridId, GridVisible, LAYERS, LAYERS_COUNT, LayerIndex},
    tile::{self, TileRegistry},
};

const WIREFRAME_HEIGHT: f32 = 100.3;
const INFO_HEIGHT: f32 = 100.2;
const IDS_HEIGHT: f32 = 100.1;

pub struct UiDebugSettingsPlugin;

impl Plugin for UiDebugSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_grids_ui);
        app.add_systems(
            Update,
            (
                (
                    draw_grid_wireframe,
                    update_physics_config,
                    update_render_config,
                )
                    .run_if(resource_changed::<UiDebugSettings>),
                draw_grid_tile_ids.run_if(
                    resource_changed::<UiDebugSettings>
                        .or(resource_changed::<TileRegistry>)
                        .or(component_changed::<GridId>),
                ),
                draw_grid_info.run_if(timeout(Duration::from_millis(100))),
            ),
        )
        .add_observer(on_add_tilemap_insert_cache)
        .insert_resource(UiDebugSettings {
            show_layers: [true; LAYERS_COUNT],
            wall_shadow: true,
            wall_border: true,
            floor_blending: true,
            ..default()
        });
    }
}

#[derive(Default, Copy, Clone, Resource)]
struct UiDebugSettings {
    show_ids: bool,
    show_grid: bool,
    show_info: bool,
    show_layers: [bool; LAYERS_COUNT],
    floor_blending: bool,
    wall_shadow: bool,
    wall_border: bool,
    show_colliders: bool,
}

#[derive(Component)]
struct DrawGridsUi;

fn list_layers() -> SpawnIter<impl Iterator<Item = impl Bundle>> {
    SpawnIter(LAYERS.into_iter().map(|l| {
        (
            Name::new(format!("Layer {l:?}")),
            checkbox((Checked,), Spawn(Text::new(format!("{l:?}")))),
            observe(
                move |change: On<ValueChange<bool>>,
                      mut commands: Commands,
                      mut config: ResMut<UiDebugSettings>| {
                    config.show_layers[l as usize] = change.value;
                    if change.value {
                        commands.entity(change.source).insert(Checked);
                    } else {
                        commands.entity(change.source).remove::<Checked>();
                    }
                },
            ),
        )
    }))
}

fn spawn_debug_grids_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug Options"),
        window(
            WindowConfig {
                title: "[Debug] Draw Grids".to_string(),
                right: px(1.0),
                bottom: px(1.0),
                ..default()
            },
            (
                Name::new("Body Content"),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![(
                    Name::new("Row"),
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(10.0),
                        ..default()
                    },
                    children![
                        (
                            Name::new("Left"),
                            Node {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            children![

                                (
                                    checkbox((), Spawn(Text::new("IDs"))),
                                    observe(
                                        |change: On<ValueChange<bool>>,
                                         mut commands: Commands,
                                         mut config: ResMut<UiDebugSettings>| {
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
                                    checkbox((), Spawn(Text::new("Grid"))),
                                    observe(
                                        |change: On<ValueChange<bool>>,
                                         mut commands: Commands,
                                         mut config: ResMut<UiDebugSettings>| {
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
                                    checkbox((), Spawn(Text::new("Info"))),
                                    observe(
                                        |change: On<ValueChange<bool>>,
                                         mut commands: Commands,
                                         mut config: ResMut<UiDebugSettings>| {
                                            config.show_info = change.value;
                                            if config.show_info {
                                                commands.entity(change.source).insert(Checked);
                                            } else {
                                                commands.entity(change.source).remove::<Checked>();
                                            }
                                        }
                                    ),
                                ),
                                (
                                    checkbox((Checked, ), Spawn(Text::new("Floor Blending"))),
                                    observe(
                                        |change: On<ValueChange<bool>>,
                                         mut commands: Commands,
                                         mut config: ResMut<UiDebugSettings>| {
                                            config.floor_blending = change.value;
                                            if config.floor_blending {
                                                commands.entity(change.source).insert(Checked);
                                            } else {
                                                commands.entity(change.source).remove::<Checked>();
                                            }
                                        }
                                    ),
                                ),
                                (
                                    checkbox((Checked,), Spawn(Text::new("Wall Shadow"))),
                                    observe(
                                        |change: On<ValueChange<bool>>,
                                         mut commands: Commands,
                                         mut config: ResMut<UiDebugSettings>| {
                                            config.wall_shadow = change.value;
                                            if config.wall_shadow {
                                                commands.entity(change.source).insert(Checked);
                                            } else {
                                                commands.entity(change.source).remove::<Checked>();
                                            }
                                        }
                                    ),
                                ),
                                (
                                    checkbox((Checked,), Spawn(Text::new("Wall Border"))),
                                    observe(
                                        |change: On<ValueChange<bool>>,
                                         mut commands: Commands,
                                         mut config: ResMut<UiDebugSettings>| {
                                            config.wall_border = change.value;
                                            if config.wall_border {
                                                commands.entity(change.source).insert(Checked);
                                            } else {
                                                commands.entity(change.source).remove::<Checked>();
                                            }
                                        }
                                    ),
                                ),
                            ]
                        ),
                        (
                            Name::new("Right"),
                            Node {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            Children::spawn((
                                Spawn(Text::new("Layers")),
                                list_layers(),
                                Spawn(spacer(5.0)),
                                Spawn((
                                    checkbox((), Spawn(Text::new("Show Colliders"))),
                                    observe(
                                        |change: On<ValueChange<bool>>,
                                         mut commands: Commands,
                                         mut config: ResMut<UiDebugSettings>| {
                                            config.show_colliders = change.value;
                                            if config.show_colliders {
                                                commands.entity(change.source).insert(Checked);
                                            } else {
                                                commands.entity(change.source).remove::<Checked>();
                                            }
                                        }
                                    ),
                                )),
                                Spawn(spacer(5.0)),
                                Spawn(Text::new("Fx FPS Multiplier")),
                                Spawn((
                                    slider(
                                        SliderProps {
                                            value: 1.0,
                                            min: 0.0,
                                            max: 2.0,
                                        },
                                        (SliderStep(0.1), SliderPrecision(1))
                                    ),
                                    observe(slider_self_update),
                                    observe(|change: On<ValueChange<f32>>, mut commands: Commands|{
                                        commands.insert_resource(FxFpsMultiplier(change.value));
                                    }),
                                )),
                            ))
                        )
                    ]
                )],
            ),
        ),
        DrawGridsUi,
    ));
}

fn format_tile_info(index: usize, ids: &GridId, elevations: &GridElevation) -> String {
    let id_or_space = |layer| match *ids[layer][index] {
        id @ 0..u16::MAX => id as i32,
        u16::MAX => -1,
    };

    let floor = id_or_space(LayerIndex::Floor);
    let wall = id_or_space(LayerIndex::Wall);
    let roof = id_or_space(LayerIndex::Roof);

    format!(
        "{floor:03},{wall:03},{roof:03}\nele: {:.02}",
        *elevations[index]
    )
}

fn tile_info_bundle(index: usize, info: String) -> impl Bundle {
    let tile_size = tile::SIZE.as_vec2();
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
                    font_size: 8.0,
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

fn draw_grid_info(
    tilemap: Single<(
        &GridVisible,
        &GridId,
        &GridElevation,
        &mut DrawGridInfoCache,
    )>,
    config: Res<UiDebugSettings>,
    mut commands: Commands,
) {
    let (grid_visible, grid_id, grid_elevation, mut cache) = tilemap.into_inner();

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
    if grid_visible.is_empty() || grid_visible.iter().filter(|t| t.is_visible()).count() > 512 {
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
                    let entity = parent.spawn(tile_info_bundle(index, info)).id();
                    cache.entities[index] = Some(entity);
                }
            });
    });

    despawn
        .into_iter()
        .for_each(|e| commands.entity(e).despawn());
}

fn draw_grid_wireframe(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut entities: Local<Vec<Entity>>,
    config: Res<UiDebugSettings>,
) {
    debug!("Drawing grid wireframe");

    entities
        .drain(..)
        .for_each(|e| commands.entity(e).despawn());

    if !config.show_grid {
        return;
    }

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
                .with_scale(tile::SIZE.as_vec2().extend(1.0)),
        ))
        .id();

    entities.push(entity);
}

fn draw_grid_tile_ids(
    grid: Single<&GridId>,
    registry: Res<TileRegistry>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut entities: Local<Vec<Entity>>,
    config: Res<UiDebugSettings>,
) {
    entities
        .drain(..)
        .for_each(|e| commands.entity(e).despawn());

    if !config.show_ids {
        return;
    }

    debug!("Drawing grid tile ids");

    let layer = &grid[LayerIndex::Floor];

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
            Transform::from_xyz(0.0, 0.0, IDS_HEIGHT).with_scale(tile::SIZE.as_vec2().extend(1.0)),
        ))
        .id();

    entities.push(entity);
}

fn update_render_config(
    config: Res<UiDebugSettings>,
    cache: Single<&TilemapCache, With<Tilemap>>,
    q_layers: Query<(&mut Visibility, &LayerIndex)>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
) {
    let Some(material) = materials.get_mut(cache.material.id()) else {
        return;
    };

    let mut mat_config = material.config.unwrap_or_default();

    mat_config.disable_floor_blending = !config.floor_blending;
    mat_config.wall_hide_outline = !config.wall_border;
    mat_config.wall_hide_shadow = !config.wall_shadow;

    material.config = Some(mat_config);

    for (mut visibility, layer) in q_layers {
        *visibility = if config.show_layers[*layer as usize] {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn update_physics_config(config: Res<UiDebugSettings>, mut store: ResMut<GizmoConfigStore>) {
    store.config_mut::<PhysicsGizmos>().0.enabled = config.show_colliders;
}
