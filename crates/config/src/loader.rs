use std::marker::PhantomData;

use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext},
    prelude::*,
    reflect::{GetTypeRegistration, TypeRegistry, serde::TypedReflectDeserializer},
};

use crate::ConfigAssetLoaderError;

#[derive(Deref, DerefMut)]
pub struct ConfigParserContext<'a, 'ctx>(&'a mut LoadContext<'ctx>);

impl<'a, 'ctx> ConfigParserContext<'a, 'ctx> {
    pub fn deserialie_ron<T>(&self, bytes: &[u8]) -> Result<T, ConfigAssetLoaderError>
    where
        T: Reflect + GetTypeRegistration + FromReflect,
    {
        use ron::extensions::Extensions;
        use serde::de::DeserializeSeed;

        let opts = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);

        let mut registry = TypeRegistry::new();
        registry.register_derived_types();

        let registration = <T as GetTypeRegistration>::get_type_registration();
        let mut deserializer = ron::de::Deserializer::from_bytes_with_options(bytes, &opts)?;
        let reflect_deserializer = TypedReflectDeserializer::new(&registration, &registry);
        let deserialized = reflect_deserializer.deserialize(&mut deserializer)?;

        let Some(asset) = <T as FromReflect>::from_reflect(&*deserialized) else {
            return Err(ConfigAssetLoaderError::Reflect);
        };

        Ok(asset)
    }

    pub async fn deserialize_file<T>(
        &mut self,
        path: impl Into<AssetPath<'_>>,
    ) -> Result<T, ConfigAssetLoaderError>
    where
        T: Reflect + GetTypeRegistration + FromReflect,
    {
        let bytes = self.0.read_asset_bytes(path).await?;
        self.deserialie_ron(&bytes)
    }
}

pub trait ConfigParser: Asset {
    type Config: Reflect + GetTypeRegistration + FromReflect;

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

        let config: T::Config = context.deserialie_ron(&bytes)?;
        let asset = T::from_config(config, context).await?;

        Ok(asset)
    }
}
