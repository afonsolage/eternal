use bevy::{ecs::system::SystemParam, prelude::*};
use eternal_config::{
    ConfigAssetLoaderError,
    loader::{ConfigAssetPlugin, ConfigAssetUpdated, ConfigParser},
    noise::NoiseStackConfig,
};

use crate::noise::stack::BoxedNoiseFn;

mod send_worley;
mod stack;
pub(crate) use stack::NoiseStack;

pub(crate) struct NoisePlugin;

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AtlasNoise>()
            .add_plugins(ConfigAssetPlugin::<NoiseStack>::default())
            .add_observer(on_config_asset_updated)
            .add_message::<NoiseChanged>()
            .add_systems(Startup, setup);
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
    assets: Res<'w, Assets<NoiseStack>>,
}

impl<'w> Noises<'w> {
    pub fn atlas(&self) -> BoxedNoiseFn {
        self.assets
            .get(self.atlas.0.id())
            .expect("atlas functon should be called only when it it is ready (is_ready() == true)")
            .main()
    }

    pub fn is_ready(&self) -> bool {
        self.assets.get(self.atlas.0.id()).is_some()
    }

    pub fn get_noise(&self, id: AssetId<NoiseStack>) -> Option<BoxedNoiseFn> {
        self.assets.get(id).map(|stack| stack.main())
    }
}

#[derive(Default, Resource)]
struct AtlasNoise(Handle<NoiseStack>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AtlasNoise(asset_server.load("config/procgen/atlas.ron")));
}

fn on_config_asset_updated(
    asset_updated: On<ConfigAssetUpdated<NoiseStack>>,
    mut writer: MessageWriter<NoiseChanged>,
    noises: Noises,
) {
    let &ConfigAssetUpdated(id) = asset_updated.event();

    if id == noises.atlas.0.id() {
        debug!("Noise atlas changed!");
        writer.write(NoiseChanged(NoiseType::Atlas));
    } else {
        debug!("Noise map changed!");
        writer.write(NoiseChanged(NoiseType::Map));
    }
}

impl ConfigParser for NoiseStack {
    type Config = NoiseStackConfig;

    async fn from_config(
        config: Self::Config,
        _load_context: eternal_config::loader::ConfigParserContext<'_, '_>,
    ) -> Result<Self, eternal_config::ConfigAssetLoaderError>
    where
        Self: Sized,
    {
        NoiseStack::parse_tree(config).map_err(|e| ConfigAssetLoaderError::Error(Box::new(e)))
    }
}
