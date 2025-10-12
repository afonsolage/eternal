use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use crate::{
    player::PlayerCamera,
    world::{
        grid::{self, GridId, LayerIndex},
        tile::{self, TileId},
    },
};

pub struct DebugCameraPlugin;

impl Plugin for DebugCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, ((zoom, pan).run_if(camera_enabled), toggle_camera));
    }
}

#[derive(Default, Component)]
struct DebugCamera {
    zoom_speed: f32,
    pan_speed: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug Camera"),
        DebugCamera {
            zoom_speed: 0.2,
            pan_speed: 0.8,
        },
        Camera2d,
        Camera {
            is_active: false,
            ..default()
        },
    ));
}

fn camera_enabled(camera: Single<&Camera, With<DebugCamera>>) -> bool {
    camera.is_active
}

fn on_map_click(release: On<Pointer<Release>>, mut grid: Single<&mut GridId>) {
    if !matches!(release.button, PointerButton::Primary) {
        return;
    }

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
}

#[allow(clippy::type_complexity)]
fn toggle_camera(
    input: Res<ButtonInput<KeyCode>>,
    debug_singleton: Single<(Entity, &mut Camera), (With<DebugCamera>, Without<PlayerCamera>)>,
    player_singleton: Single<(Entity, &mut Camera), (With<PlayerCamera>, Without<DebugCamera>)>,
    map: Single<Entity, With<GridId>>,
    mut commands: Commands,
    mut cache: Local<Option<Entity>>,
) {
    if input.just_pressed(KeyCode::KeyP) {
        let (debug_entity, mut debug_cam) = debug_singleton.into_inner();
        let (player_entity, mut player_cam) = player_singleton.into_inner();

        let is_active = !debug_cam.is_active;

        debug_cam.is_active = is_active;
        player_cam.is_active = !is_active;

        let map_entity = map.into_inner();

        if debug_cam.is_active {
            commands.entity(debug_entity).insert(IsDefaultUiCamera);
            commands.entity(player_entity).remove::<IsDefaultUiCamera>();

            let obs_entity = commands
                .spawn(Observer::new(on_map_click).with_entity(map_entity))
                .id();

            *cache = Some(obs_entity);
        } else {
            commands.entity(debug_entity).remove::<IsDefaultUiCamera>();
            commands.entity(player_entity).insert(IsDefaultUiCamera);

            if let Some(obs_entity) = cache.take() {
                commands.entity(obs_entity).despawn();
            }
        }
    }
}

fn zoom(
    mut scroll_msgs: MessageReader<MouseWheel>,
    singleton: Single<(&mut Projection, &DebugCamera)>,
) {
    let (mut projection, camera) = singleton.into_inner();

    let Projection::Orthographic(ortho) = projection.as_mut() else {
        return;
    };

    for ev in scroll_msgs.read() {
        let zoom_delta = ev.y * camera.zoom_speed;
        ortho.scale -= zoom_delta;
        ortho.scale = ortho.scale.clamp(0.1, 10.0);
    }
}

fn pan(
    mut motion_msgs: MessageReader<MouseMotion>,
    input: Res<ButtonInput<MouseButton>>,
    singleton: Single<(&mut Transform, &Projection, &DebugCamera)>,
) {
    if !input.pressed(MouseButton::Middle) {
        return;
    }

    let (mut transform, projection, camera) = singleton.into_inner();
    for ev in motion_msgs.read() {
        let scale = if let Projection::Orthographic(ortho) = projection {
            ortho.scale
        } else {
            1.0
        };

        transform.translation.x -= ev.delta.x * camera.pan_speed * scale;
        transform.translation.y += ev.delta.y * camera.pan_speed * scale;
    }
}
