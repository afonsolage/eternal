use std::time::Duration;

use bevy::{math::U16Vec2, platform::collections::HashMap, prelude::*};

use crate::{
    config::tile::{TileConfig, TileConfigList},
    run_conditions::timeout,
    world::{
        actions::ActionsPlugin,
        genesis::GenesisPlugin,
        grid::{GridId, GridIdChanged, GridVisible, LAYERS},
        physics::PhysicsPlugin,
        renderer::{MapRendererPlugin, tilemap::Tilemap},
        tile::{TileId, TileInfo, TileRegistry, TileVisible},
    },
};

mod actions;
pub mod genesis;
pub mod grid;
pub mod physics;
pub mod renderer;
pub mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MapRendererPlugin,
            PhysicsPlugin,
            GenesisPlugin,
            ActionsPlugin,
        ))
        .init_resource::<TileRegistry>()
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            (
                process_tile_info_list.run_if(on_message::<AssetEvent<TileConfigList>>),
                update_tile_visibility.run_if(timeout(Duration::from_millis(100))),
                update_tile_ids.run_if(timeout(Duration::from_millis(100))),
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
    commands.spawn((Name::new("Map"), tilemap, GridId::new(), GridVisible::new()));
}

#[derive(Resource)]
#[expect(unused, reason = "The handle needs to be hold somewhere")]
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

fn update_tile_visibility(
    mut grid: Single<&mut GridVisible>,
    q_camera: Query<
        (&Camera, &GlobalTransform),
        Or<(Changed<GlobalTransform>, Changed<Projection>)>,
    >,
    mut last_rect: Local<Rect>,
) {
    let Some((camera, camera_transform)) = q_camera.iter().find(|&(c, _)| c.is_active) else {
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

fn update_tile_ids(mut grid: Single<&mut GridId>, mut commands: Commands) {
    for layer_index in LAYERS {
        let queue = grid[layer_index].drain_queue();

        // Avoid triggering change detection
        if queue.is_empty() {
            continue;
        }

        let (positions, values): (Vec<_>, Vec<_>) = queue.into_iter().unzip();

        let layer = &mut grid[layer_index];
        for (&U16Vec2 { x, y }, value) in positions.iter().zip(values) {
            layer.set(x, y, value);
        }

        commands.trigger(GridIdChanged(layer_index, positions));
    }
}
