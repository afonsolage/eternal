#![forbid(unsafe_code)]

use bevy::prelude::*;
use bevy::window::PresentMode;

use eternal_client::ClientPlugin;

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
        .add_plugins(ClientPlugin)
        .run();
}
