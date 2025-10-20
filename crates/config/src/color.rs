use bevy::{color::Srgba, prelude::Deref, reflect::Reflect};

#[derive(Debug, Clone, Copy, Deref, Reflect)]
pub struct HexColor(pub Srgba);

impl From<HexColor> for bevy::color::Color {
    fn from(HexColor(srgba): HexColor) -> Self {
        srgba.into()
    }
}
