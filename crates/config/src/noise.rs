use bevy::prelude::*;
use serde::Deserialize;

use crate::server::{ConfigServerPlugin, FromConfig};

pub(crate) struct NoiseStackConfigPlugin;
impl Plugin for NoiseStackConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ConfigServerPlugin::<NoiseStackConfig>::default(),));
    }
}

#[derive(Debug, Copy, Clone, Default, Reflect, Deserialize)]
pub enum WorleyConfigReturnType {
    #[default]
    Value,
    Distance,
}

#[derive(Debug, Clone, Reflect, Deserialize)]
pub enum NoiseFnConfig {
    Fbm {
        seed: u32,
        frequency: f64,
        octaves: usize,
        lacunarity: f64,
        persistence: f64,
    },
    Billow {
        seed: u32,
        frequency: f64,
        octaves: usize,
        lacunarity: f64,
        persistence: f64,
    },
    Worley {
        seed: u32,
        frequency: f64,
        return_type: WorleyConfigReturnType,
    },
    Curve {
        source: String,
        control_points: Vec<(f64, f64)>,
    },
    ScaleBias {
        source: String,
        scale: f64,
        bias: f64,
    },
    Min {
        source_1: String,
        source_2: String,
    },
    Max {
        source_1: String,
        source_2: String,
    },
    Multiply {
        source_1: String,
        source_2: String,
    },
    Add {
        source_1: String,
        source_2: String,
    },
    Clamp {
        source: String,
        bounds: (f64, f64),
    },
    Turbulence {
        source: String,
        seed: u32,
        frequency: f64,
        power: f64,
        roughness: usize,
    },
    Select {
        source_1: String,
        source_2: String,
        control: String,
        bounds: (f64, f64),
        falloff: f64,
    },
    Terrace {
        source: String,
        control_points: Vec<f64>,
    },
    RidgedMulti {
        seed: u32,
        frequency: f64,
        lacunarity: f64,
        octaves: usize,
    },
    Constant(f64),
    Blend {
        source_1: String,
        source_2: String,
        control: String,
    },
    Exponent {
        source: String,
        exponent: f64,
    },
    Alias(String),
}

impl NoiseFnConfig {
    pub fn dependencies(&self) -> Vec<&str> {
        match self {
            // No Sources
            NoiseFnConfig::Fbm { .. }
            | NoiseFnConfig::RidgedMulti { .. }
            | NoiseFnConfig::Worley { .. }
            | NoiseFnConfig::Billow { .. }
            | NoiseFnConfig::Constant(..) => vec![],
            // Single Sources
            NoiseFnConfig::Curve { source, .. }
            | NoiseFnConfig::ScaleBias { source, .. }
            | NoiseFnConfig::Turbulence { source, .. }
            | NoiseFnConfig::Terrace { source, .. }
            | NoiseFnConfig::Exponent { source, .. }
            | NoiseFnConfig::Clamp { source, .. }
            | NoiseFnConfig::Alias(source) => {
                vec![source]
            }
            // Two sources
            NoiseFnConfig::Min { source_1, source_2 }
            | NoiseFnConfig::Max { source_1, source_2 }
            | NoiseFnConfig::Add { source_1, source_2 }
            | NoiseFnConfig::Multiply { source_1, source_2 } => {
                vec![source_1, source_2]
            }
            // Three sources
            NoiseFnConfig::Select {
                source_1,
                source_2,
                control: source_3,
                ..
            }
            | NoiseFnConfig::Blend {
                source_1,
                source_2,
                control: source_3,
            } => vec![source_1, source_2, source_3],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NoiseConfigError {
    #[error("Failed to load noise stack: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to load noise stack: {0}")]
    RonDeserialize(#[from] ron::error::SpannedError),
    #[error("Failed to load noise stack: {0}")]
    Deserialize(#[from] ron::error::Error),
}

#[derive(Default, Reflect, Clone, Deserialize, Deref)]
pub struct NoiseStackConfig(pub Vec<(String, NoiseFnConfig)>);

impl FromConfig for NoiseStackConfig {
    type InnerType = Vec<(String, NoiseFnConfig)>;

    fn from_inner<'a>(inner: Self::InnerType) -> Self {
        Self(inner)
    }
}
