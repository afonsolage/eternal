use bevy::{ecs::system::SystemParam, prelude::*};
use eternal_config::{
    loader::{ConfigAssetPlugin, ConfigAssetUpdated, ConfigParser, ConfigParserContext},
    tile::TileConfig,
};
use eternal_grid::tile::TileId;

use crate::noise::NoiseStack;

pub(crate) struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ConfigAssetPlugin::<BiomeRegistry>::default(),
            ConfigAssetPlugin::<BiomePallet>::default(),
        ))
        .add_observer(on_biome_registry_config_updated)
        .add_observer(on_biome_pallet_config_updated)
        .init_resource::<BiomeRegistry>()
        .add_systems(Startup, setup);
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
        let tiles_config: Vec<TileConfig> = load_context
            .deserialize_config_from_file("config/tiles.ron")
            .await?;

        let pallet = config
            .into_iter()
            .filter_map(|(height, tile_name)| {
                let Some(index) = tiles_config
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

fn on_biome_registry_config_updated(
    updated: On<ConfigAssetUpdated<BiomeRegistry>>,
    mut commands: Commands,
    biomes: Res<Assets<BiomeRegistry>>,
) {
    let &ConfigAssetUpdated(id) = updated.event();
    let Some(biomes) = biomes.get(id) else {
        return;
    };

    debug!("Updating biomes resource!");

    commands.insert_resource(biomes.clone());
}

fn on_biome_pallet_config_updated(
    _updated: On<ConfigAssetUpdated<BiomePallet>>,
    mut biome_registry: ResMut<BiomeRegistry>,
) {
    // just to trigger the change detection
    biome_registry.set_changed();
}
