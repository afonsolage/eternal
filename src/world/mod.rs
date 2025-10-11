use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    config::tile::{TileConfig, TileConfigList},
    world::{
        genesis::GenesisPlugin,
        grid::{GridId, GridIdChanged, GridVisible, LayerIndex},
        physics::PhysicsPlugin,
        renderer::{MapRendererPlugin, tilemap::Tilemap},
        tile::{TileId, TileInfo, TileRegistry, TileVisible},
    },
};

pub mod genesis;
pub mod grid;
pub mod physics;
pub mod renderer;
pub mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MapRendererPlugin, PhysicsPlugin, GenesisPlugin))
            .init_resource::<TileRegistry>()
            .add_systems(Startup, setup)
            .add_systems(
                PreUpdate,
                (
                    process_tile_info_list.run_if(on_message::<AssetEvent<TileConfigList>>),
                    update_tile_visibility.run_if(time_passed(0.5)),
                    trigger_grid_changed,
                ),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tilemap = Tilemap {
        atlas_texture: asset_server.load("sheets/terrain.png"),
        atlas_dims: UVec2::new(4, 4),
    };

    commands.insert_resource(TileInfoHandle(asset_server.load("config/tiles.ron")));
    commands
        .spawn((Name::new("Map"), tilemap, GridId::new(), GridVisible::new()))
        .observe(
            |release: On<Pointer<Release>>, mut grid: Single<&mut GridId>| {
                let Some(pos) = release.hit.position else {
                    return;
                };

                let tile_pos = pos.xy().as_u16vec2() / tile::SIZE;

                if tile_pos.x as u32 > grid::DIMS.x || tile_pos.y as u32 > grid::DIMS.y {
                    return;
                }

                let current = *grid[LayerIndex::WALL].get(tile_pos.x, tile_pos.y);
                grid[LayerIndex::WALL].set(tile_pos.x, tile_pos.y, TileId::default());

                debug!("Changing {current:?} to none at {tile_pos}");
            },
        );
}

fn time_passed(t: f32) -> impl FnMut(Local<f32>, Res<Time>) -> bool {
    move |mut timer: Local<f32>, time: Res<Time>| {
        // Tick the timer
        *timer += time.delta_secs();
        // Return true if the timer has passed the time
        *timer >= t
    }
}

#[derive(Resource)]
#[allow(unused)]
struct TileInfoHandle(Handle<TileConfigList>);

fn process_tile_info_list(
    mut msg_reader: MessageReader<AssetEvent<TileConfigList>>,
    assets: Res<Assets<TileConfigList>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for msg in msg_reader.read() {
        debug!("Event: {msg:?}");
        if let &AssetEvent::Added { id } | &AssetEvent::Modified { id } = msg
            && let Some(tile_config_list) = assets.get(id)
        {
            let map = tile_config_list
                .0
                .iter()
                .enumerate()
                .map(|(idx, config)| {
                    let TileConfig {
                        name,
                        kind,
                        atlas,
                        atlas_index,
                        map_color,
                        blend_tech,
                    } = config;

                    let info = TileInfo {
                        name: name.clone().into(),
                        kind: *kind,
                        atlas: asset_server.load(atlas),
                        atlas_index: *atlas_index,
                        map_color: map_color.0,
                        blend_tech: blend_tech.unwrap_or_default(),
                    };

                    let id = TileId::new(idx as u16);
                    (id, info)
                })
                .chain(std::iter::once((TileId::new(u16::MAX), tile::NONE_INFO)))
                .collect::<HashMap<_, _>>();

            debug!("Loaded tile info list: {map:?}");

            commands.insert_resource(TileRegistry::new(map));
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_tile_visibility(
    mut grid: Single<&mut GridVisible>,
    q_camera: Query<
        (&Camera, &GlobalTransform),
        Or<(Changed<GlobalTransform>, Changed<Projection>)>,
    >,
    mut last_rect: Local<Rect>,
) {
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    let Ok(top_left) = camera.viewport_to_world_2d(camera_transform, Vec2::ZERO) else {
        return;
    };
    let Some(viewport_size) = camera.logical_viewport_size() else {
        return;
    };
    let Ok(bottom_right) = camera.viewport_to_world_2d(camera_transform, viewport_size) else {
        return;
    };

    let rect = Rect::new(top_left.x, bottom_right.y, bottom_right.x, top_left.y);

    if *last_rect == rect {
        return;
    }

    *last_rect = rect;

    let min_tile = (rect.min / tile::SIZE.as_vec2())
        .clamp(Vec2::ZERO, grid::DIMS.as_vec2() - Vec2::ONE)
        .as_u16vec2();
    let max_tile = (rect.max / tile::SIZE.as_vec2())
        .clamp(Vec2::ZERO, grid::DIMS.as_vec2() - Vec2::ONE)
        .as_u16vec2();

    grid.fill(TileVisible::default());

    for y in min_tile.y..=max_tile.y {
        for x in min_tile.x..=max_tile.x {
            grid.set(x, y, TileVisible::visible());
        }
    }
}

fn trigger_grid_changed(changed: Query<(), Changed<GridId>>, mut commands: Commands) {
    if !changed.is_empty() {
        commands.trigger(GridIdChanged)
    }
}
