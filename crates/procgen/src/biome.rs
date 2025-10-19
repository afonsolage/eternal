use bevy::prelude::*;
use eternal_config::{
    loader::{ConfigAssetLoader, ConfigParser, ConfigParserContext},
    tile::TileConfigList,
};
use eternal_grid::tile::TileId;

use crate::noise::NoiseStack;

pub(crate) struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Biomes>()
            .init_asset::<BiomePallet>()
            .init_asset_loader::<ConfigAssetLoader<Biomes>>()
            .init_asset_loader::<ConfigAssetLoader<BiomePallet>>()
            .add_systems(Startup, setup);
    }
}

#[derive(Default, Asset, Debug, Clone, Reflect)]
pub struct BiomePallet(Vec<(f32, TileId)>);

#[derive(Debug, Clone, Reflect)]
pub struct Biome {
    name: String,
    terrain_noise: Handle<NoiseStack>,
    terrain_pallet: Handle<BiomePallet>,
}

#[derive(Asset, Default, Debug, Clone, Resource, Reflect)]
pub struct Biomes(Vec<Biome>);

#[expect(unused, reason = "I need to keep the handle on the resource")]
#[derive(Resource)]
struct BiomesHandle(Handle<Biomes>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BiomesHandle(asset_server.load("config/procgen/biomes.ron")));
}

impl ConfigParser for Biomes {
    type Config = Vec<(String, String)>;

    async fn from_config(
        config: Self::Config,
        mut load_context: ConfigParserContext<'_, '_>,
    ) -> Result<Self, eternal_config::ConfigAssetLoaderError>
    where
        Self: Sized,
    {
        let biomes = config
            .into_iter()
            .map(|(name, folder)| Biome {
                name,
                terrain_noise: load_context.load(format!("config/procgen/{folder}/terrain.ron")),
                terrain_pallet: load_context.load(format!("config/procgen/{folder}/pallet.ron")),
            })
            .collect();

        debug!("Biomes loaded: {biomes:?}");

        Ok(Biomes(biomes))
    }
}

impl ConfigParser for BiomePallet {
    type Config = Vec<(f32, String)>;

    async fn from_config(
        config: Self::Config,
        mut load_context: ConfigParserContext<'_, '_>,
    ) -> Result<Self, eternal_config::ConfigAssetLoaderError>
    where
        Self: Sized,
    {
        let tiles_config: TileConfigList =
            load_context.deserialize_file("config/tiles.ron").await?;

        let pallet = config
            .into_iter()
            .filter_map(|(height, tile_name)| {
                let Some(index) = tiles_config
                    .0
                    .iter()
                    .position(|config| config.name == tile_name)
                else {
                    error!("Tile {tile_name} not found on tile config list!");
                    return None;
                };

                Some((height, TileId::new(index as u16)))
            })
            .collect();

        debug!("Pallet loaded: {pallet:?}");

        Ok(BiomePallet(pallet))
    }
}
