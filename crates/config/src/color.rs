use bevy::{color::Srgba, prelude::Deref, reflect::Reflect};

#[derive(Debug, Clone, Deref, Reflect)]
pub struct HexColor(pub String);

impl From<HexColor> for bevy::color::Color {
    fn from(HexColor(hex): HexColor) -> Self {
        Srgba::hex(&hex).unwrap_or_default().into()
    }
}

impl From<&HexColor> for bevy::color::Srgba {
    fn from(value: &HexColor) -> Self {
        Srgba::hex(&value.0).unwrap_or_default()
    }
}
