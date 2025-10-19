use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (zoom, pan));
    }
}
#[derive(Default, Component)]
struct DebugCamera {
    zoom_speed: f32,
    pan_speed: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug Cam"),
        DebugCamera {
            zoom_speed: 0.05,
            pan_speed: 0.8,
        },
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
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
        ortho.scale = ortho.scale.clamp(0.01, 0.5);
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
