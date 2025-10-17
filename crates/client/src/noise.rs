#![expect(unused, reason = "I'll revisit this whole class later on")]
use bevy::{math::vec2, prelude::*};
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

pub(crate) struct Noise {
    continentalness: Fbm<Perlin>,
    curve: Vec<Vec2>,
}

impl Noise {
    pub fn new(seed: u32) -> Self {
        // TODO: Move this to a config per-biome
        let continentalness = Fbm::<Perlin>::new(seed)
            .set_frequency(0.03)
            .set_octaves(3)
            .set_lacunarity(0.10);

        let curve = vec![
            vec2(-1.0, 50.0),
            vec2(0.3, 100.0),
            vec2(0.4, 150.0),
            vec2(1.0, 150.0),
        ];

        Noise {
            continentalness,
            curve,
        }
    }

    fn lerp(&self, t: f32) -> i32 {
        assert!(self.curve.len() >= 2);

        let min = self.curve.first().unwrap();
        let max = self.curve.last().unwrap();

        assert!(t >= min.x);
        assert!(t <= max.x);

        for segment in self.curve.windows(2) {
            let begin = segment[0];
            let end = segment[1];

            if t >= begin.x && t <= end.x {
                // Normalize 't' within the segment
                let normalized_t = (t - begin.x) / (end.x - begin.x);

                // Linear interpolation
                return (begin + (end - begin) * normalized_t).y as i32;
            }
        }

        unreachable!()
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
