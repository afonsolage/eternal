use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    config::tile::{TileConfig, TileConfigList},
    world::{
        genesis::generate_grids,
        grid::Grid,
        renderer::{MapRendererPlugin, tilemap::Tilemap},
        tile::{TileId, TileInfo, TileRegistry, TileVisible},
    },
};

pub mod genesis;
pub mod grid;
pub mod renderer;
pub mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MapRendererPlugin)
            .add_systems(Startup, setup)
            .add_systems(
                PreUpdate,
                (
                    process_tile_info_list,
                    update_tile_visibility.run_if(time_passed(0.5)),
                ),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let grids = generate_grids();
    let tilemap = Tilemap {
        atlas_texture: asset_server.load("sheets/terrain.png"),
        atlas_dims: UVec2::new(4, 4),
        tile_size: Vec2::new(32.0, 32.0),
    };

    commands.insert_resource(TileInfoHandle(asset_server.load("config/tiles.ron")));
    commands.spawn((Name::new("Map"), tilemap, grids, Grid::<TileVisible>::new()));
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
    q_tiles: Query<(&Tilemap, &mut Grid<TileVisible>)>,
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

    debug!("Updating tile visibility: {rect:?}");

    for (tilemap, mut grid) in q_tiles {
        let min_tile = (rect.min / tilemap.tile_size)
            .clamp(Vec2::ZERO, grid::DIMS.as_vec2() - Vec2::ONE)
            .as_u16vec2();
        let max_tile = (rect.max / tilemap.tile_size)
            .clamp(Vec2::ZERO, grid::DIMS.as_vec2() - Vec2::ONE)
            .as_u16vec2();

        debug!("Updating tile visibility: {min_tile:?} - {max_tile:?}");

        grid.fill(TileVisible::default());

        for y in min_tile.y..=max_tile.y {
            for x in min_tile.x..=max_tile.x {
                grid.set(x, y, TileVisible::visible());
            }
        }
    }
}
