use std::{any::TypeId, marker::PhantomData};

use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext},
    prelude::*,
    reflect::{GetTypeRegistration, TypeRegistry, serde::TypedReflectDeserializer},
};

use crate::ConfigAssetLoaderError;

pub struct ConfigAssetPlugin<T>(PhantomData<T>);
impl<T> Default for ConfigAssetPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Event)]
pub struct ConfigAssetUpdated<T: Asset>(pub AssetId<T>);

impl<T> ConfigAssetPlugin<T>
where
    T: Asset + Send + Sync + 'static,
{
    fn trigger_config_events(mut reader: MessageReader<AssetEvent<T>>, mut commands: Commands) {
        for &msg in reader.read() {
            match msg {
                AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                    commands.trigger(ConfigAssetUpdated(id));
                }
                _ => continue,
            }
        }
    }
}

impl<T> Plugin for ConfigAssetPlugin<T>
where
    T: Asset + ConfigParser,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<T>()
            .init_asset_loader::<ConfigAssetLoader<T>>()
            .add_systems(
                Update,
                Self::trigger_config_events.run_if(on_message::<AssetEvent<T>>),
            );
    }
}

#[derive(Deref, DerefMut)]
pub struct ConfigParserContext<'a, 'ctx>(&'a mut LoadContext<'ctx>);

impl<'a, 'ctx> ConfigParserContext<'a, 'ctx> {
    pub async fn deserialize_config_from_file<T>(
        &mut self,
        path: impl Into<AssetPath<'_>>,
    ) -> Result<T, ConfigAssetLoaderError>
    where
        T: Reflect + GetTypeRegistration + FromReflect + TypePath,
    {
        let bytes = self.0.read_asset_bytes(path).await?;
        ConfigAssetLoader::<T>::deserialize_config(&bytes)
    }
}

pub trait ConfigParser: Asset {
    type Config: Reflect + GetTypeRegistration + FromReflect + TypePath;

    fn from_config(
        config: Self::Config,
        load_context: ConfigParserContext<'_, '_>,
    ) -> impl std::future::Future<Output = Result<Self, ConfigAssetLoaderError>> + Send
    where
        Self: Sized;
}

pub struct ConfigAssetLoader<T>(PhantomData<T>);
impl<T> Default for ConfigAssetLoader<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> ConfigAssetLoader<T> {
    pub fn deserialize_config<C>(bytes: &[u8]) -> Result<C, ConfigAssetLoaderError>
    where
        C: Reflect + GetTypeRegistration + FromReflect + TypePath,
    {
        use ron::extensions::Extensions;
        use serde::de::DeserializeSeed;

        let opts = ron::Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);

        let mut registry = TypeRegistry::new();
        registry.register_derived_types();
        registry.register::<C>();

        let registration = registry
            .get(TypeId::of::<C>())
            .expect("Just registered it!");

        let mut deserializer = ron::de::Deserializer::from_bytes_with_options(bytes, &opts)?;

        let reflect_deserializer = TypedReflectDeserializer::new(registration, &registry);
        let deserialized = reflect_deserializer.deserialize(&mut deserializer)?;

        assert!(deserialized.as_partial_reflect().represents::<C>());

        let Some(asset) = <C as FromReflect>::from_reflect(deserialized.as_partial_reflect())
        else {
            error!("{:?}", &*deserialized);
            return Err(ConfigAssetLoaderError::Reflect(
                "Failed to convert from reflect.",
            ));
        };

        Ok(asset)
    }
}

impl<T> AssetLoader for ConfigAssetLoader<T>
where
    T: ConfigParser,
{
    type Asset = T;

    type Settings = ();

    type Error = ConfigAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let context = ConfigParserContext(load_context);

        let config: T::Config = Self::deserialize_config(&bytes)?;
        let asset = T::from_config(config, context).await?;

        Ok(asset)
    }
}

#[cfg(test)]
mod tests {
    use crate::tile::{TileConfig, TileConfigList};

    use super::*;

    #[test]
    fn loader() {
        let input_path = format!("{}/config/tiles.ron", env!("ASSETS_PATH"));
        let bytes = std::fs::read(input_path).unwrap();
        let _config: Vec<TileConfig> =
            ConfigAssetLoader::<TileConfigList>::deserialize_config(&bytes).unwrap();
    }
}
