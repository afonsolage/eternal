use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::player::Player;

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_player)
            .add_systems(Update, zoom.run_if(camera_enabled));
    }
}

#[derive(Component)]
pub struct PlayerCamera {
    zoom_speed: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self { zoom_speed: 0.2 }
    }
}

fn on_add_player(add: On<Add, Player>, mut commands: Commands) {
    commands.entity(add.entity).with_child((
        Camera2d,
        Name::new("Player Cam"),
        IsDefaultUiCamera,
        PlayerCamera::default(),
    ));
}

fn camera_enabled(camera: Single<&Camera, With<PlayerCamera>>) -> bool {
    camera.is_active
}

fn zoom(
    mut scroll_msgs: MessageReader<MouseWheel>,
    singleton: Single<(&mut Projection, &PlayerCamera)>,
) {
    let (mut projection, camera) = singleton.into_inner();

    let Projection::Orthographic(ortho) = projection.as_mut() else {
        return;
    };

    for ev in scroll_msgs.read() {
        let zoom_delta = ev.y * camera.zoom_speed;
        ortho.scale -= zoom_delta;
        ortho.scale = ortho.scale.clamp(0.1, 1.0);
    }
}
