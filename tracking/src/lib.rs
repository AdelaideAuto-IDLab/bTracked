extern crate nalgebra as na;
extern crate nalgebra_glm as glm;
extern crate ncollide2d;
extern crate particle_filter;
extern crate probability;
extern crate rand;
extern crate rayon;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate stats;

pub mod filter_runner;
mod signal;
mod util;
pub mod geometry;
pub mod distance_field;

use std::collections::HashMap;

pub use filter_runner::{Particle, FilterRunner};
pub use signal::MeasurementModel;

pub type Polygon = Vec<[f32; 2]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub filter: FilterConfig,
    pub geometry: GeometryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub num_particles: usize,
    pub reinit_threshold: f32,
    pub reinit_ratio: f32,
    pub update_rate_ms: u64,
    pub stationary: ModelConfig,
    pub motion: ModelConfig,
    pub speed: f32,
    pub turn_rate_mean: f32,
    pub turn_rate_stddev: f32,
    pub signal: Vec<SignalConfig>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelConfig {
    pub kinematic_noise: f32,
    pub turn_rate_noise: f32,
    pub pose_noise: f32,
    pub transition_prob: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConfig {
    pub alpha: f32,
    pub beta: f32,
    pub noise: f32,
    pub gain_table: GainTable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GainTable {
    pub horizontal: Vec<f32>,
    pub vertical: Vec<f32>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

fn one() -> f32 { 1.0 }
fn version_0() -> String { "0".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryConfig {
    #[serde(default = "version_0")]
    pub version: String,
    pub boundary: Rect,
    #[serde(default = "one")]
    pub scale: f32,
    pub walls: Vec<[[f32; 2]; 2]>,
    #[serde(default)]
    pub obstacles: Vec<Rect>,
    #[serde(default)]
    pub zones: HashMap<String, Polygon>,
    #[serde(default)]
    pub signal_sources: HashMap<String, SignalSource>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SignalSource {
    pub position: glm::Vec3,
    pub direction: glm::Vec3,
    #[serde(default)]
    pub model_id: usize,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub source: SignalSource,
    pub rssi: i16,
}

#[derive(Debug, Clone, Serialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub stddev: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub point: Point,
    pub zone: Option<String>,
}
