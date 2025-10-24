use bevy::prelude::*;

use crate::server::{ConfigServerPlugin, FromConfig};

pub(crate) struct FloraConfigPlugin;
impl Plugin for FloraConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ConfigServerPlugin::<FloraRegistryConfig>::default(),));
    }
}

#[derive(Reflect, Default, Debug, Clone)]
pub struct FloraConfig {
    pub name: String,
    pub tile: String,
    pub threshold: f32,
    pub spacing: f32,
    pub elevation_range: Option<(f32, f32)>,
    pub allowed_terrains: Vec<String>,
}

#[derive(Reflect, Default, Debug, Clone, Deref)]
pub struct FloraRegistryConfig(pub Vec<FloraConfig>);

impl FromConfig for FloraRegistryConfig {
    type InnerType = Vec<FloraConfig>;

    fn from_inner<'a>(inner: Self::InnerType) -> Self {
        Self(inner)
    }
}

#[cfg(test)]
mod tests {
    use crate::server::deserialize_config;

    pub use super::*;

    const RON: &str = r#"
[
    (
        name: "TREE",
        tile: "TREE_WALL",
        threshold: 0.5,
        spacing: 3.0,
        elevation_range: Some((0.0, 0.5)),
        allowed_terrains: ["GRASS"],
    ),
]
    "#;

    #[test]
    fn deserialize() {
        // Arrange
        // Act
        let registry: FloraRegistryConfig =
            deserialize_config::<FloraRegistryConfig>(RON.as_bytes());

        // Assert
        let config = registry.first().unwrap();
        assert_eq!(&config.name, "TREE");
        assert_eq!(&config.tile, "TREE_WALL");
        assert_eq!(config.threshold, 0.5);
        assert_eq!(config.spacing, 3.0);
        assert_eq!(config.elevation_range, Some((0.0, 0.5)));
        assert_eq!(config.allowed_terrains, vec!["GRASS"]);
    }
}
