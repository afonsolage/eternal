use bevy::{ecs::system::SystemParam, prelude::*};

use crate::noise::stack::{BoxedNoiseFn, NoiseStack, NoiseStackLoader};

mod send_worley;
mod stack;

pub struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AtlasNoise>()
            .init_asset::<NoiseStack>()
            .add_message::<NoiseChanged>()
            .init_asset_loader::<NoiseStackLoader>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                send_noise_changed_messages.run_if(on_message::<AssetEvent<NoiseStack>>),
            );
    }
}

#[derive(Clone, Copy)]
pub enum NoiseType {
    Atlas,
}

#[derive(Message, Clone, Copy)]
pub struct NoiseChanged(pub NoiseType);

#[derive(SystemParam)]
pub struct Noises<'w> {
    atlas: Res<'w, AtlasNoise>,
    assets: Res<'w, Assets<NoiseStack>>,
}

impl<'w> Noises<'w> {
    pub fn atlas(&self) -> BoxedNoiseFn {
        self.assets
            .get(self.atlas.0.id())
            .expect("AtlasNoise should always exists")
            .main()
    }
}

#[derive(Default, Resource)]
struct AtlasNoise(Handle<NoiseStack>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AtlasNoise(asset_server.load("config/procgen/atlas.ron")));
}

fn send_noise_changed_messages(
    mut reader: MessageReader<AssetEvent<NoiseStack>>,
    mut writer: MessageWriter<NoiseChanged>,
    noises: Noises,
) {
    for &msg in reader.read() {
        match msg {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if id == noises.atlas.0.id() {
                    debug!("Noise atlas changed!");
                    writer.write(NoiseChanged(NoiseType::Atlas));
                }
            }
            _ => continue,
        }
    }
}
