use std::{any::TypeId, marker::PhantomData};

use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext},
    prelude::*,
    reflect::{GetTypeRegistration, TypeRegistry, serde::TypedReflectDeserializer},
};

use crate::ConfigAssetLoaderError;

#[derive(Deref, DerefMut)]
pub struct ConfigParserContext<'a, 'ctx>(&'a mut LoadContext<'ctx>);

impl<'a, 'ctx> ConfigParserContext<'a, 'ctx> {
    pub fn deserialie_config<T>(&self, bytes: &[u8]) -> Result<T, ConfigAssetLoaderError>
    where
        T: Reflect + GetTypeRegistration + FromReflect + TypePath,
    {
        use ron::extensions::Extensions;
        use serde::de::DeserializeSeed;

        let opts = ron::Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES);

        let mut registry = TypeRegistry::new();
        registry.register_derived_types();

        let registration =
            registry
                .get(TypeId::of::<T>())
                .ok_or(ConfigAssetLoaderError::Reflect(
                    "TypeId not found on registry",
                ))?;
        let mut deserializer = ron::de::Deserializer::from_bytes_with_options(bytes, &opts)?;

        let reflect_deserializer = TypedReflectDeserializer::new(registration, &registry);
        let deserialized = reflect_deserializer.deserialize(&mut deserializer)?;

        assert!(deserialized.as_partial_reflect().represents::<T>());

        let Some(asset) = <T as FromReflect>::from_reflect(deserialized.as_partial_reflect())
        else {
            error!("{:?}", &*deserialized);
            return Err(ConfigAssetLoaderError::Reflect(
                "Failed to convert from reflect.",
            ));
        };

        Ok(asset)
    }

    pub async fn deserialize_config_from_file<T>(
        &mut self,
        path: impl Into<AssetPath<'_>>,
    ) -> Result<T, ConfigAssetLoaderError>
    where
        T: Reflect + GetTypeRegistration + FromReflect + TypePath,
    {
        let bytes = self.0.read_asset_bytes(path).await?;
        self.deserialie_config(&bytes)
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

#[derive(Default)]
pub struct ConfigAssetLoader<T>(PhantomData<T>);

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

        let config: T::Config = context.deserialie_config(&bytes)?;
        let asset = T::from_config(config, context).await?;

        Ok(asset)
    }
}
