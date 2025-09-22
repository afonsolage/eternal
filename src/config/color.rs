use bevy::{color::Srgba, prelude::Deref};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deref)]
pub struct HexColor(pub Srgba);

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex: &str = Deserialize::deserialize(deserializer)?;

        Srgba::hex(hex)
            .map(HexColor)
            .map_err(serde::de::Error::custom)
    }
}

impl From<HexColor> for bevy::color::Color {
    fn from(HexColor(srgba): HexColor) -> Self {
        srgba.into()
    }
}
