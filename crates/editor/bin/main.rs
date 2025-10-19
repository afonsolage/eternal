use bevy::{prelude::*, window::PresentMode};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),))
        .add_plugins(eternal_editor::EditorPlugin)
        .run();
}
