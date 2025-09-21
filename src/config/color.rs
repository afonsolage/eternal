use bevy::{color::Srgba, prelude::Deref};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deref)]
pub struct ConfigColor(Srgba);

impl<'de> Deserialize<'de> for ConfigColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex: &str = Deserialize::deserialize(deserializer)?;

        Srgba::hex(hex)
            .map(ConfigColor)
            .map_err(serde::de::Error::custom)
    }
}

impl From<ConfigColor> for bevy::color::Color {
    fn from(ConfigColor(srgba): ConfigColor) -> Self {
        srgba.into()
    }
}
