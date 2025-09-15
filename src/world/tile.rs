use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Deref, Reflect)]
#[repr(transparent)]
pub struct TileType(pub u16);

impl TileType {
    pub fn color(self) -> Color {
        match self.0 {
            0 => Color::BLACK,
            _ => Color::WHITE,
        }
    }
}
