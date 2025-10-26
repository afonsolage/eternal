use bevy::{platform::collections::HashMap, reflect::Reflect};
use eternal_config::noise::{NoiseFnConfig, NoiseStackConfig, WorleyConfigReturnType};
use noise::{
    Add, Billow, Blend, Clamp, Constant, Curve, Exponent, Fbm, Max, Min, MultiFractal, Multiply,
    NoiseFn, Perlin, RidgedMulti, ScaleBias, Seedable, Select, Terrace, Turbulence, core::worley,
};

use super::send_worley::SendWorley;

type BoxedNoiseFn = Box<dyn NoiseFn<f64, 2> + Send + 'static + Sync>;

fn from_worley_config(spec: WorleyConfigReturnType) -> worley::ReturnType {
    match spec {
        WorleyConfigReturnType::Value => worley::ReturnType::Value,
        WorleyConfigReturnType::Distance => worley::ReturnType::Distance,
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum NoiseStackParserError {
    #[error("Failed to load noise stack: Noise stack is empty")]
    Empty,
    #[error("Failed to load noise stack: No main layer was found")]
    NoMain,
    #[error("Failed to build noise stack: Layer {0} was not found")]
    NotFound(String),
    #[error("Failed to load noise stack: Missing dependencies on layer.")]
    DepSpec,
    #[error("Duplicated layer names detected: {0}")]
    DuplicatedNames(String),
}

#[derive(Default, Reflect)]
pub struct NoiseStack {
    specs: HashMap<String, NoiseFnConfig>,
    #[reflect(ignore)]
    main: Option<BoxedNoiseFn>,
}

impl std::fmt::Debug for NoiseStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoiseStack")
            .field("specs", &self.specs)
            .finish()
    }
}

impl Clone for NoiseStack {
    fn clone(&self) -> Self {
        Self {
            specs: self.specs.clone(),
            main: None,
        }
    }
}

impl NoiseStack {
    pub(crate) fn from_config(
        specs: &NoiseStackConfig,
    ) -> Result<NoiseStack, NoiseStackParserError> {
        if specs.is_empty() {
            return Err(NoiseStackParserError::Empty);
        }

        if !specs.iter().any(|(name, _)| name == "main") {
            return Err(NoiseStackParserError::NoMain);
        }

        if let Some((name, _)) = specs
            .iter()
            .fold(HashMap::new(), |mut map, (name, _)| {
                *map.entry(name).or_insert(0usize) += 1;

                if *map.get(name).unwrap() > 1 {
                    bevy::log::warn!("Duplicated spec name: {name}");
                }

                map
            })
            .into_iter()
            .find(|(_, v)| *v > 1)
        {
            return Err(NoiseStackParserError::DuplicatedNames(name.to_string()));
        }

        let mut invalid = false;
        for (name, spec) in specs.iter() {
            for dep in spec.dependencies() {
                if !specs.iter().any(|(name, _)| name == dep) {
                    bevy::log::warn!("Dependency {dep} not found on spec {name}");
                    invalid = true;
                }
            }
        }

        if invalid {
            Err(NoiseStackParserError::DepSpec)
        } else {
            bevy::log::debug!("Noise tree loaded.");
            let specs = specs.0.clone().into_iter().collect();
            let main = Self::build(&specs, "main")?;
            Ok(NoiseStack {
                specs,
                main: Some(main),
            })
        }
    }

    fn build(
        specs: &HashMap<String, NoiseFnConfig>,
        name: &str,
    ) -> Result<BoxedNoiseFn, NoiseStackParserError> {
        let Some(spec) = specs.get(name) else {
            return Err(NoiseStackParserError::NotFound(name.to_string()));
        };

        let noise_fn: BoxedNoiseFn = match spec {
            NoiseFnConfig::Fbm {
                seed,
                frequency,
                octaves,
                lacunarity,
                persistence,
            } => {
                let fbm = Fbm::<Perlin>::new(*seed)
                    .set_frequency(*frequency)
                    .set_octaves(*octaves)
                    .set_lacunarity(*lacunarity)
                    .set_persistence(*persistence);
                Box::new(fbm)
            }
            NoiseFnConfig::Worley {
                seed,
                frequency,
                return_type,
            } => {
                let worley = SendWorley::new(*seed)
                    .set_frequency(*frequency)
                    .set_return_type(from_worley_config(*return_type));
                Box::new(worley)
            }
            NoiseFnConfig::Billow {
                seed,
                frequency,
                octaves,
                lacunarity,
                persistence,
            } => {
                let billow = Billow::<Perlin>::new(*seed)
                    .set_frequency(*frequency)
                    .set_octaves(*octaves)
                    .set_lacunarity(*lacunarity)
                    .set_persistence(*persistence);
                Box::new(billow)
            }
            NoiseFnConfig::Curve {
                source,
                control_points,
            } => {
                let source = Self::build(specs, source)?;
                let curve = control_points
                    .iter()
                    .copied()
                    .fold(Curve::new(source), |c, (input, output)| {
                        c.add_control_point(input, output)
                    });
                Box::new(curve)
            }
            NoiseFnConfig::ScaleBias {
                source,
                scale,
                bias,
            } => {
                let source = Self::build(specs, source)?;
                Box::new(ScaleBias::new(source).set_scale(*scale).set_bias(*bias))
            }
            NoiseFnConfig::Min { source_1, source_2 } => {
                let source_1 = Self::build(specs, source_1)?;
                let source_2 = Self::build(specs, source_2)?;
                Box::new(Min::new(source_1, source_2))
            }
            NoiseFnConfig::Max { source_1, source_2 } => {
                let source_1 = Self::build(specs, source_1)?;
                let source_2 = Self::build(specs, source_2)?;
                Box::new(Max::new(source_1, source_2))
            }
            NoiseFnConfig::Multiply { source_1, source_2 } => {
                let source_1 = Self::build(specs, source_1)?;
                let source_2 = Self::build(specs, source_2)?;
                Box::new(Multiply::new(source_1, source_2))
            }
            NoiseFnConfig::Add { source_1, source_2 } => {
                let source_1 = Self::build(specs, source_1)?;
                let source_2 = Self::build(specs, source_2)?;
                Box::new(Add::new(source_1, source_2))
            }
            NoiseFnConfig::Clamp { source, bounds } => {
                let source = Self::build(specs, source)?;
                Box::new(Clamp::new(source).set_bounds(bounds.0, bounds.1))
            }
            NoiseFnConfig::Exponent { source, exponent } => {
                let source = Self::build(specs, source)?;
                Box::new(Exponent::new(source).set_exponent(*exponent))
            }
            NoiseFnConfig::Turbulence {
                source,
                seed,
                frequency,
                power,
                roughness,
            } => {
                let source = Self::build(specs, source)?;
                let turbulence = Turbulence::<_, Perlin>::new(source)
                    .set_seed(*seed)
                    .set_frequency(*frequency)
                    .set_power(*power)
                    .set_roughness(*roughness);
                Box::new(turbulence)
            }
            NoiseFnConfig::Select {
                source_1,
                source_2,
                control,
                bounds,
                falloff,
            } => {
                let source_1 = Self::build(specs, source_1)?;
                let source_2 = Self::build(specs, source_2)?;
                let control = Self::build(specs, control)?;
                let select = Select::new(source_1, source_2, control)
                    .set_bounds(bounds.0, bounds.1)
                    .set_falloff(*falloff);
                Box::new(select)
            }
            NoiseFnConfig::Terrace {
                source,
                control_points: control_ponts,
            } => {
                let source = Self::build(specs, source)?;
                let terrace = control_ponts
                    .iter()
                    .copied()
                    .fold(Terrace::new(source), |t, p| t.add_control_point(p));

                Box::new(terrace)
            }
            NoiseFnConfig::RidgedMulti {
                seed,
                frequency,
                lacunarity,
                octaves,
            } => {
                let ridged_multi = RidgedMulti::<Perlin>::new(*seed)
                    .set_frequency(*frequency)
                    .set_lacunarity(*lacunarity)
                    .set_octaves(*octaves);
                Box::new(ridged_multi)
            }
            NoiseFnConfig::Constant(value) => Box::new(Constant::new(*value)),
            NoiseFnConfig::Blend {
                source_1,
                source_2,
                control,
            } => {
                let source_1 = Self::build(specs, source_1)?;
                let source_2 = Self::build(specs, source_2)?;
                let control = Self::build(specs, control)?;
                let blend = Blend::new(source_1, source_2, control);
                Box::new(blend)
            }
            NoiseFnConfig::Alias(source) => Self::build(specs, source)?,
        };

        Ok(noise_fn)
    }

    pub fn get(&self, x: f32, y: f32) -> f32 {
        self.main
            .as_ref()
            .expect("Main to be built")
            .get([x as f64, y as f64]) as f32
    }

    pub fn is_ready(&self) -> bool {
        self.main.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    // pub(crate) fn rebuild(&mut self) -> Result<(), NoiseStackParserError> {
    //     self.main = Some(Self::build(&self.specs, "main")?);
    //     Ok(())
    // }
}
