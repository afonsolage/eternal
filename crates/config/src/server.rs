use std::marker::PhantomData;

use bevy::{
    asset::{AssetLoader, AssetPath, UntypedAssetId},
    ecs::system::SystemParam,
    prelude::*,
    reflect::Reflectable,
};

use crate::ConfigAssetLoaderError;

pub trait FromConfig: Reflectable + Send + Sync + 'static {
    type InnerType: Reflectable + FromReflect;

    fn from_inner(inner: Self::InnerType) -> Self;
}

#[derive(Asset, Reflect)]
struct ConfigAsset<C: FromConfig>(C);

pub(crate) struct ConfigServerPlugin<T>(PhantomData<T>);
impl<T> Default for ConfigServerPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Resource)]
struct AssetAdded<C>(PhantomData<C>);
impl<C> Default for AssetAdded<C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Plugin for ConfigServerPlugin<T>
where
    T: FromConfig,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigAsset<T>>()
            .init_asset_loader::<ConfigAssetLoader<T>>()
            .init_resource::<AssetAdded<T>>()
            .add_systems(Update, trigger_config_asset_updated::<T>);
    }
}

#[derive(EntityEvent)]
pub struct ConfigAssetUpdated {
    entity: Entity,
    id: UntypedAssetId,
}
impl ConfigAssetUpdated {
    pub fn id(&self) -> UntypedAssetId {
        self.id
    }
}

#[derive(Component, Debug)]
struct ConfigHandler<C: FromConfig>(Handle<ConfigAsset<C>>);

#[derive(SystemParam)]
pub struct ConfigServer<'w, 's> {
    asset_server: Res<'w, AssetServer>,
    commands: Commands<'w, 's>,
}

impl<'w, 's> ConfigServer<'w, 's> {
    pub fn load<'a, 'p, C>(&'a mut self, path: impl Into<AssetPath<'p>>) -> EntityCommands<'a>
    where
        C: FromConfig,
    {
        self.commands
            .spawn(ConfigHandler::<C>(self.asset_server.load(path)))
    }
}

#[derive(SystemParam)]
pub struct Configs<'w, C>
where
    C: FromConfig,
{
    assets: Res<'w, Assets<ConfigAsset<C>>>,
}

impl<'w, C> Configs<'w, C>
where
    C: FromConfig,
{
    pub fn get(&self, id: UntypedAssetId) -> Option<&C> {
        self.assets.get(id.typed()).map(|asset| &asset.0)
    }
}

fn trigger_config_asset_updated<C: FromConfig>(
    mut reader: MessageReader<AssetEvent<ConfigAsset<C>>>,
    mut commands: Commands,
    handlers: Query<(Entity, &ConfigHandler<C>)>,
) {
    for &msg in reader.read() {
        match msg {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let Some((entity, _)) = handlers.iter().find(|(_, handler)| handler.0.id() == id)
                else {
                    error!("Config handler not found for asset {id}");
                    continue;
                };

                commands.entity(entity).trigger(|e| ConfigAssetUpdated {
                    entity: e,
                    id: id.untyped(),
                });
            }
            _ => continue,
        }
    }
}

struct ConfigAssetLoader<T>(PhantomData<T>);
impl<T> Default for ConfigAssetLoader<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> AssetLoader for ConfigAssetLoader<T>
where
    T: FromConfig,
    T::InnerType: FromReflect + Reflectable,
{
    type Asset = ConfigAsset<T>;

    type Settings = ();

    type Error = ConfigAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> std::result::Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let config: T::InnerType = deserialize_inner_type(&bytes)?;

        let asset = T::from_inner(config);

        Ok(ConfigAsset(asset))
    }
}

pub(crate) fn deserialize_inner_type<C>(bytes: &[u8]) -> Result<C, ConfigAssetLoaderError>
where
    C: Reflectable + FromReflect,
{
    use ron::extensions::Extensions;
    use serde::de::DeserializeSeed;

    let opts = ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);

    let mut registry = bevy::reflect::TypeRegistry::new();
    registry.register_derived_types();
    registry.register::<C>();

    let registration = registry
        .get(std::any::TypeId::of::<C>())
        .expect("Just registered it!");

    let mut deserializer = ron::de::Deserializer::from_bytes_with_options(bytes, &opts)?;

    let reflect_deserializer =
        bevy::reflect::serde::TypedReflectDeserializer::new(registration, &registry);
    let deserialized = reflect_deserializer.deserialize(&mut deserializer)?;

    assert!(deserialized.as_partial_reflect().represents::<C>());

    let Some(config) = <C as FromReflect>::from_reflect(deserialized.as_partial_reflect()) else {
        return Err(ConfigAssetLoaderError::Reflect(format!(
            "Failed to convert from reflect. Deserialized: {:?}",
            &*deserialized
        )));
    };

    Ok(config)
}

#[cfg(test)]
pub(crate) fn deserialize_config<C>(bytes: &[u8]) -> C
where
    C: FromConfig,
{
    let inner = deserialize_inner_type(bytes).unwrap();

    C::from_inner(inner)
}
