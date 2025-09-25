use std::{borrow::Cow, marker::PhantomData};

use bevy::prelude::*;
use serde::Deserialize;

pub const NONE_INFO: TileInfo = TileInfo {
    name: Cow::Borrowed("NONE"),
    kind: TileKind::Terrain,
    atlas: Handle::Uuid(AssetId::<Image>::DEFAULT_UUID, PhantomData),
    atlas_index: u16::MAX,
    map_color: Srgba::NONE,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Deref, Reflect)]
#[repr(transparent)]
pub struct TileId(u16);

impl TileId {
    pub fn new(id: u16) -> Self {
        Self(id)
    }
}

impl Default for TileId {
    fn default() -> Self {
        Self(u16::MAX)
    }
}

#[derive(Debug, Default, Clone, Copy, Reflect, Deserialize)]
pub enum TileKind {
    #[default]
    Terrain,
}

#[derive(Debug, Default, Clone, Reflect)]
pub struct TileInfo {
    pub name: Cow<'static, str>,
    pub kind: TileKind,
    pub atlas: Handle<Image>,
    pub atlas_index: u16,
    pub map_color: Srgba,
}

#[derive(Debug, Default, Clone, Reflect, Deref, DerefMut, Resource)]
pub struct TileInfos(Vec<TileInfo>);

impl TileInfos {
    pub fn new(list: Vec<TileInfo>) -> Self {
        Self(list)
    }

    pub fn get(&self, id: TileId) -> &TileInfo {
        &self.0[id.0 as usize]
    }
}
