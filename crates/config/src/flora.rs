use bevy::prelude::*;

use crate::server::{ConfigServerPlugin, FromConfig};

pub(crate) struct FloraConfigPlugin;
impl Plugin for FloraConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ConfigServerPlugin::<FloraRegistryConfig>::default(),
            ConfigServerPlugin::<FloraSpawnRegistryConfig>::default(),
        ));
    }
}

#[derive(Reflect, Debug, Clone)]
pub enum CollisionShape {
    Circle(f32),
}

#[derive(Reflect, Default, Debug, Clone)]
pub struct FloraConfig {
    pub name: String,
    pub sprite_sheet: String,
    pub sprite_region: UVec2,
    pub sprite_size: UVec2,
    pub anchor: UVec2,
    pub collision_shape: Option<CollisionShape>,
}

#[derive(Reflect, Default, Debug, Clone, Deref)]
pub struct FloraRegistryConfig(pub Vec<FloraConfig>);

impl FromConfig for FloraRegistryConfig {
    type InnerType = Vec<FloraConfig>;

    fn from_inner<'a>(inner: Self::InnerType) -> Self {
        Self(inner)
    }
}

#[derive(Reflect, Default, Debug, Clone)]
pub struct FloraSpawnConfig {
    pub name: String,
    pub flora: String,
    pub threshold: f32,
    pub wall_spacing: u8,
    pub floor_spacing: u8,
    pub elevation_range: Option<(f32, f32)>,
    pub allowed_terrains: Vec<String>,
}

#[derive(Reflect, Default, Debug, Clone, Deref)]
pub struct FloraSpawnRegistryConfig(pub Vec<FloraSpawnConfig>);

impl FromConfig for FloraSpawnRegistryConfig {
    type InnerType = Vec<FloraSpawnConfig>;

    fn from_inner<'a>(inner: Self::InnerType) -> Self {
        Self(inner)
    }
}

#[cfg(test)]
mod tests {
    use crate::server::deserialize_config;

    pub use super::*;

    #[test]
    fn deserialize_spawn_registry() {
        // Arrange
        const SPAWN: &str = r#"
[
    (
        name: "TREE",
        flora: "TREE",
        threshold: 0.5,
        wall_spacing: 1,
        floor_spacing: 3,
        elevation_range: Some((0.0, 0.5)),
        allowed_terrains: ["GRASS"],
    ),
]
    "#;
        // Act
        let registry: FloraSpawnRegistryConfig =
            deserialize_config::<FloraSpawnRegistryConfig>(SPAWN.as_bytes());

        // Assert
        let config = registry.first().unwrap();
        assert_eq!(&config.name, "TREE");
        assert_eq!(&config.flora, "TREE");
        assert_eq!(config.threshold, 0.5);
        assert_eq!(config.wall_spacing, 1);
        assert_eq!(config.floor_spacing, 3);
        assert_eq!(config.elevation_range, Some((0.0, 0.5)));
        assert_eq!(config.allowed_terrains, vec!["GRASS"]);
    }

    #[test]
    fn deserialize_registry() {
        // Arrange
        const FLORA: &str = r#"
[
    (
        name: "TREE",
        sprite_sheet: "sheets/flora.png",
        sprite_region: (0, 0),
        sprite_size: (34, 57),
        anchor: (10, 9),
        collision_shape: Some(Circle(0.5)),
    ),
]
    "#;
        // Act
        let registry: FloraRegistryConfig =
            deserialize_config::<FloraRegistryConfig>(FLORA.as_bytes());

        // Assert
        let config = registry.first().unwrap();
        assert_eq!(&config.name, "TREE");
        assert_eq!(&config.sprite_sheet, "sheets/flora.png");
        assert_eq!(config.sprite_region, UVec2::new(0, 0));
        assert_eq!(config.sprite_size, UVec2::new(34, 57));
        assert_eq!(config.anchor, UVec2::new(10, 9));
        assert!(
            config
                .collision_shape
                .as_ref()
                .is_some_and(|shape| matches!(shape, CollisionShape::Circle(0.5)))
        );
    }
}
