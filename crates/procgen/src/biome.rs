use bevy::{ecs::system::SystemParam, prelude::*};
use eternal_config::{
    loader::{ConfigAssetLoader, ConfigParser, ConfigParserContext},
    tile::TileConfigList,
};
use eternal_grid::tile::TileId;

use crate::noise::NoiseStack;

pub(crate) struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BiomeRegistry>()
            .init_asset::<BiomePallet>()
            .init_asset_loader::<ConfigAssetLoader<BiomeRegistry>>()
            .init_asset_loader::<ConfigAssetLoader<BiomePallet>>()
            .init_resource::<BiomeRegistry>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                update_biomes_res.run_if(on_message::<AssetEvent<BiomeRegistry>>),
            );
    }
}

#[derive(Default, Asset, Debug, Clone, Reflect)]
pub struct BiomePallet(Vec<(f32, TileId)>);

impl BiomePallet {
    pub fn collapse(&self, value: f32) -> TileId {
        for &(threshould, tile_id) in &self.0 {
            if value < threshould {
                return tile_id;
            }
        }

        Default::default()
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct Biome {
    pub name: String,
    pub terrain_noise: Handle<NoiseStack>,
    pub terrain_pallet: Handle<BiomePallet>,
}

#[derive(Asset, Default, Debug, Clone, Resource, Reflect, Deref)]
struct BiomeRegistry(Vec<Biome>);
#[derive(SystemParam)]
pub struct Biomes<'w> {
    registry: Res<'w, BiomeRegistry>,
    pallets: Res<'w, Assets<BiomePallet>>,
}

impl<'w> Biomes<'w> {
    pub fn is_ready(&self) -> bool {
        !self.registry.is_empty()
    }

    pub fn get_biome(&self, name: &str) -> Option<&Biome> {
        self.registry.iter().find(|b| b.name == name)
    }

    pub fn get_pallet(&self, biome: &str) -> Option<&BiomePallet> {
        self.registry.iter().find_map(|b| {
            if b.name == biome {
                self.pallets.get(b.terrain_pallet.id())
            } else {
                None
            }
        })
    }
}

#[expect(unused, reason = "I need to keep the handle on the resource")]
#[derive(Resource)]
struct BiomesHandle(Handle<BiomeRegistry>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BiomesHandle(asset_server.load("config/procgen/biomes.ron")));
}

impl ConfigParser for BiomeRegistry {
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

        Ok(BiomeRegistry(biomes))
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

fn update_biomes_res(
    mut reader: MessageReader<AssetEvent<BiomeRegistry>>,
    mut commands: Commands,
    biomes: Res<Assets<BiomeRegistry>>,
) {
    for &msg in reader.read() {
        match msg {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let Some(biomes) = biomes.get(id) else {
                    continue;
                };

                debug!("Updating biomes resource!");

                commands.insert_resource(biomes.clone());
            }
            _ => continue,
        }
    }
}
