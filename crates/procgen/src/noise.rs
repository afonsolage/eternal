use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

pub(crate) struct Noise {
    continentalness: Fbm<Perlin>,
}

impl Noise {
    pub fn new(seed: u32) -> Self {
        // TODO: Move this to a config per-biome
        let continentalness = Fbm::<Perlin>::new(seed)
            .set_frequency(0.03)
            .set_octaves(3)
            .set_lacunarity(0.10);

        Noise { continentalness }
    }

    pub fn get(&self, x: f32, y: f32) -> f32 {
        self.continentalness.get([x as f64, y as f64]) as f32
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self::new(42)
    }
}
