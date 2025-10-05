use std::{borrow::Cow, marker::PhantomData};

use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserialize;

pub const NONE_INFO: TileInfo = TileInfo {
    name: Cow::Borrowed("NONE"),
    kind: TileKind::Terrain,
    atlas: Handle::Uuid(AssetId::<Image>::DEFAULT_UUID, PhantomData),
    atlas_index: u16::MAX,
    map_color: Srgba::NONE,
    blend_tech: BlendTech::None,
};

#[derive(Debug, Default, Clone, Copy, Reflect, Deserialize)]
pub enum TileKind {
    #[default]
    Terrain,
    Wall,
}

#[derive(Debug, Default, Clone, Copy, Reflect, Deserialize)]
pub enum BlendTech {
    #[default]
    None,
    Weight(u16),
}

#[derive(Debug, Default, Clone, Reflect)]
pub struct TileInfo {
    pub name: Cow<'static, str>,
    pub kind: TileKind,
    pub atlas: Handle<Image>,
    pub atlas_index: u16,
    pub map_color: Srgba,
    pub blend_tech: BlendTech,
}

#[derive(Debug, Default, Clone, Reflect, Deref, DerefMut, Resource)]
pub struct TileRegistry(HashMap<TileId, TileInfo>);

impl TileRegistry {
    pub fn new(map: HashMap<TileId, TileInfo>) -> Self {
        Self(map)
    }
}

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

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Deref, Reflect)]
#[repr(transparent)]
pub struct TileElevation(f32);

impl TileElevation {
    pub fn new(elevation: f32) -> Self {
        Self(elevation)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Deref, Reflect)]
#[repr(transparent)]
pub struct TileVisible(bool);

impl TileVisible {
    pub fn visible() -> Self {
        Self(true)
    }

    pub fn is_visible(&self) -> bool {
        self.0
    }
}
