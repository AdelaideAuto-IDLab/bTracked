use std::f32::consts;

use glm;
use probability::distribution::{Continuous, Gaussian};
use rand::{Rng, distributions::Normal};

use super::{Measurement, SignalConfig, SignalSource};
use filter_runner::Particle;

pub struct MeasurementModel {
    models: Vec<MeasurementModelInner>
}

impl MeasurementModel {
    pub fn new(models: Vec<SignalConfig>) -> MeasurementModel {
        MeasurementModel {
            models: models.into_iter().map(|config| MeasurementModelInner::new(config)).collect()
        }
    }

    pub fn generate<R>(&self, rng: &mut R, source: &SignalSource, state: &Particle) -> Measurement
    where
        R: Rng,
    {
        self.models[source.model_id].generate(rng, source, state)
    }

    pub fn probability(&self, measurements: &[Measurement], state: Particle) -> f32 {
        let mut weight: f64 = 1.0;
        for measurement in measurements {
            weight *= self.models[measurement.source.model_id].weight(measurement, state);
        }

        weight as f32
    }

    pub fn distance_bound_for_rssi(&self, rssi: f32, upper_bound: f32) -> f32 {
        // FIXME use the model that returns the largest distance instead of just the first
        self.models[0].distance_bound_for_rssi(rssi, upper_bound)
    }
}

struct MeasurementModelInner {
    noise_model: Gaussian,
    noise_source: Normal,
    normalization_factor: f64,
    config: SignalConfig,
}

impl MeasurementModelInner {
    fn new(config: SignalConfig) -> MeasurementModelInner {
        MeasurementModelInner {
            noise_model: Gaussian::new(0.0, config.noise as f64),
            noise_source: Normal::new(0.0, config.noise as f64),
            normalization_factor: (config.noise * (2.0 * consts::PI).sqrt()) as f64,
            config,
        }
    }

    fn generate<R>(&self, rng: &mut R, source: &SignalSource, state: &Particle) -> Measurement
    where
        R: Rng,
    {
        let rssi = self.rssi_model(source, state) + rng.sample(self.noise_source) as f32;
        Measurement {
            source: *source,
            rssi: rssi as i16
        }
    }

    fn weight(&self, measurement: &Measurement, state: Particle) -> f64 {
        let expected_rssi = self.rssi_model(&measurement.source, &state);
        let difference = (expected_rssi - measurement.rssi as f32) as f64;
        self.noise_model.density(difference) * self.normalization_factor
    }

    fn rssi_model(&self, source: &SignalSource, state: &Particle) -> f32 {
        let measurement_pos = source.position.xy();
        let particle_pos = state.position.xy();

        let dist = glm::distance(&source.position, &state.position);
        self.base_model(dist) + self.directional_gain(&measurement_pos, &particle_pos, state.pose)
    }

    fn base_model(&self, dist: f32) -> f32 {
        -10.0 * self.config.alpha * (dist + 0.00001).log(10.0) + self.config.beta
    }

    fn directional_gain(&self, measurement_pos: &glm::Vec2, particle_pos: &glm::Vec2, pose: f32) -> f32 {
        let source_dir = measurement_pos - particle_pos;
        let pose_dir = glm::vec2(pose.cos(), pose.sin());

        let phi = glm::angle(&source_dir, &pose_dir);

        let gain_table = &self.config.gain_table.horizontal;
        let resolution = (gain_table.len() - 1) as f32;
        let index = (phi / (2.0 * consts::PI) * resolution).floor() as usize;

        gain_table[index]
    }

    fn distance_bound_for_rssi(&self, rssi: f32, upper_bound: f32) -> f32 {
        // Some signal models are non-invertible so we use a simple numerical method instead of
        // trying to find a closed-form solution.

        const MAX_ITERATIONS: usize = 1000;
        let step_size = upper_bound / MAX_ITERATIONS as f32;

        for i in 0..MAX_ITERATIONS {
            let distance = upper_bound - i as f32 * step_size;
            if rssi < self.base_model(distance) {
                return distance;
            }
        }

        0.0
    }
}
