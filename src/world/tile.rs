use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Deref, Reflect)]
#[repr(transparent)]
pub struct TileId(pub u16);
