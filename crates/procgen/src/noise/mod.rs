use bevy::{ecs::system::SystemParam, prelude::*};

use crate::noise::stack::{BoxedNoiseFn, NoiseStackLoader};

mod send_worley;
mod stack;
pub(crate) use stack::NoiseStack;

pub(crate) struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AtlasNoise>()
            .init_resource::<MapNoise>()
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
    Map,
}

#[derive(Message, Clone, Copy)]
pub struct NoiseChanged(pub NoiseType);

#[derive(SystemParam)]
pub struct Noises<'w> {
    atlas: Res<'w, AtlasNoise>,
    map: Res<'w, MapNoise>,
    assets: Res<'w, Assets<NoiseStack>>,
}

impl<'w> Noises<'w> {
    pub fn atlas(&self) -> BoxedNoiseFn {
        self.assets
            .get(self.atlas.0.id())
            .expect("atlas functon should be called only when it it is ready (is_ready() == true)")
            .main()
    }

    pub fn map(&self) -> BoxedNoiseFn {
        self.assets
            .get(self.map.0.id())
            .expect("map functon should be called only when it it is ready (is_ready() == true)")
            .main()
    }

    pub fn is_ready(&self) -> bool {
        self.assets.get(self.atlas.0.id()).is_some()
    }
}

#[derive(Default, Resource)]
struct AtlasNoise(Handle<NoiseStack>);

#[derive(Default, Resource)]
struct MapNoise(Handle<NoiseStack>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AtlasNoise(asset_server.load("config/procgen/atlas.ron")));
    commands.insert_resource(MapNoise(asset_server.load("config/procgen/map.ron")));
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
                } else if id == noises.map.0.id() {
                    debug!("Noise map changed!");
                    writer.write(NoiseChanged(NoiseType::Map));
                }
            }
            _ => continue,
        }
    }
}
