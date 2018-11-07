use glm::{self, vec3, Vec3};
use rand::{self, Rng, FromEntropy, rngs::SmallRng, distributions::StandardNormal};
use stats;

use particle_filter::{Filter, ParticleFilter};

use {
    signal::MeasurementModel,
    geometry::World,
    distance_field::DistanceField,
    util, TrackingConfig, FilterConfig, ModelConfig, Measurement
};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParticleMode {
    Stationary,
    Moving,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub pose: f32,
    pub turn_rate: f32,
    pub mode: ParticleMode,
}

struct SystemNoise {
    position: Vec3,
    velocity: Vec3,
    turn_rate: f32,
    pose: f32,
}

impl SystemNoise {
    fn gen(rng: &mut SmallRng, config: &ModelConfig, dt: f32) -> SystemNoise {
        let mut randn = |stddev: f32| rng.sample(StandardNormal) as f32 * stddev;

        SystemNoise {
            position: vec3(
                0.5 * dt * dt * randn(config.kinematic_noise),
                0.5 * dt * dt * randn(config.kinematic_noise),
                0.0,
            ),
            velocity: vec3(
                dt * randn(config.kinematic_noise),
                dt * randn(config.kinematic_noise),
                0.0,
            ),
            turn_rate: dt * dt * randn(config.turn_rate_noise),
            pose: dt * randn(config.pose_noise),
        }
    }
}

struct MotionModel {
    distance_field: DistanceField,
    stationary: ModelConfig,
    motion: ModelConfig,
    speed: f32,
}

fn propagate(model: &MotionModel, rng: &mut SmallRng, particle: Particle, dt: f32) -> Particle {
    let Particle { mut position, mut velocity, mut pose, mut turn_rate, mut mode } = particle;

    match mode {
        ParticleMode::Stationary => {
            let noise = SystemNoise::gen(rng, &model.stationary, dt);

            position += noise.position;
            turn_rate += noise.turn_rate;
            pose = util::wrap_angle(pose + noise.pose + dt * turn_rate);

            if rng.gen::<f32>() < model.stationary.transition_prob * dt {
                mode = ParticleMode::Moving;
                velocity = util::rand_velocity(model.speed);
                // velocity = model.speed * vec3(pose.cos(), pose.sin(), 0.0);
            }
        },
        ParticleMode::Moving => {
            let noise = SystemNoise::gen(rng, &model.motion, dt);

            let repulsion = {
                let [x, y] = model.distance_field.query(position.x, position.y);
                dt * vec3(x, y, 0.0)
            };

            if turn_rate == 0.0 {
                // Prevent degeneracy due to floating point division by zero.
                // (Note: limit x -> 0: sin(x) / x == 1)
                turn_rate = 0.00001;
            }
            let motion_model = (1.0 / turn_rate) * glm::mat3(
                (dt * turn_rate).sin(),       (dt * turn_rate).cos() - 1.0, 0.0,
                1.0 - (dt * turn_rate).cos(), (dt * turn_rate).sin(),       0.0,
                0.0,                           0.0,                         0.0,
            );

            position += motion_model * velocity - repulsion + noise.position;
            velocity = glm::rotate_z_vec3(&velocity, dt * turn_rate) - repulsion + noise.velocity;
            turn_rate += noise.turn_rate;

            let delta_p = position - particle.position;
            pose = delta_p.y.atan2(delta_p.x);

            if rng.gen::<f32>() < model.motion.transition_prob * dt {
                mode = ParticleMode::Stationary;
            }
        }
    }

    Particle { position, velocity, pose, turn_rate, mode }
}

type BoxFilter = Box<dyn Filter<Particle=Particle, Measurement=[Measurement]>>;

pub struct FilterRunner {
    filter: BoxFilter,
    config: FilterConfig,
    width: f32,
    height: f32,
}

impl FilterRunner {
    pub fn new(tracking_config: &TrackingConfig, distance_field: Option<DistanceField>) -> FilterRunner {
        let TrackingConfig { ref filter, ref geometry } = tracking_config;

        let scale = geometry.scale;
        let width = geometry.boundary.width / scale;
        let height = geometry.boundary.height / scale;

        let particles = generate_initial_particles(width, height, filter);

        let model = MotionModel {
            distance_field: {
                distance_field.unwrap_or_else(|| DistanceField::new(&World::new(geometry), 100.0))
            },
            stationary: filter.stationary,
            motion: filter.motion,
            speed: filter.speed,
        };
        let mut rng = SmallRng::from_entropy();
        let propagation_fn = move |particle: Particle, dt: f32| {
            propagate(&model, &mut rng, particle, dt)
        };

        let noise_fn = move |particle: Particle, _dt: f32| {
            particle
        };

        let measurement_model = MeasurementModel::new(filter.signal.clone());
        let weight_fn = move |particle: Particle, measurements: &[Measurement]| {
            let x = particle.position.x;
            let y = particle.position.y;
            if x < 0.0 || x > width || y < 0.0 || y > height {
                // Particle is out of bounds
                return 0.0;
            }

            measurement_model.probability(measurements, particle)
        };

        let particle_filter = ParticleFilter::new(particles, propagation_fn, noise_fn, weight_fn);
        FilterRunner {
            filter: Box::new(particle_filter) as BoxFilter,
            config: filter.clone(),
            width,
            height,
        }
    }

    pub fn step(&mut self, measurements: &[Measurement], dt: f32) -> f32 {
        let weight = self.filter.step(measurements, dt);
        if weight < self.config.reinit_threshold {
            self.reinitialize();
            return 1.0;
        }
        weight
    }

    pub fn reinitialize(&mut self) {
        let particles = generate_initial_particles(self.width, self.height, &self.config);
        self.filter.merge_particles(&particles, self.config.reinit_ratio);
    }

    pub fn get_estimate(&self) -> Particle {
        let particles = self.filter.get_particles();

        // Taking the median of each of the values individually isn't really correct here, what we
        // really want is the cluster centroid, but that is more costly to compute and in practice
        // the median works fairly well.
        macro_rules! median_of {
            ($x:expr) => {
                stats::median(particles.iter().map($x).map(|x| x as f64)).unwrap() as f32
            }
        };
        let x = median_of!(|p| p.position.x);
        let y = median_of!(|p| p.position.y);
        let dx = median_of!(|p| p.velocity.x);
        let dy = median_of!(|p| p.velocity.y);

        let pose = median_of!(|p| p.pose);
        let turn_rate = median_of!(|p| p.turn_rate);

        let stationary = particles.iter().filter(|p| p.mode == ParticleMode::Stationary).count();
        let mode = match stationary > particles.len() / 2 {
            true => ParticleMode::Stationary,
            false => ParticleMode::Moving,
        };

        Particle {
            position: vec3(x, y, 0.0),
            velocity: vec3(dx, dy, 0.0),
            pose,
            turn_rate,
            mode
        }
    }

    pub fn get_snapshot(&self, num_particles: usize) -> Vec<Particle> {
        let particles = self.filter.get_particles();

        if num_particles >= particles.len() {
            return particles.into();
        }

        let mut rng = rand::thread_rng();
        rand::seq::sample_slice(&mut rng, particles, num_particles)
    }
}

/// Generate an initial particle distribution based on the provided config
fn generate_initial_particles(width: f32, height: f32, config: &FilterConfig) -> Vec<Particle> {
    let mut rng = rand::thread_rng();
    (0..config.num_particles).map(|_| {
        let position = {
            let base = rng.gen::<[f32; 2]>();
            glm::vec3(base[0] * width, base[1] * height, 0.0)
        };
        let velocity = util::rand_velocity(config.speed);
        let mode = match rng.gen::<bool>() {
            true => ParticleMode::Stationary,
            false => ParticleMode::Moving,
        };

        let turn_rate = rng.sample(StandardNormal) as f32 * config.turn_rate_stddev + config.turn_rate_mean;

        Particle {
            position,
            velocity,
            pose: velocity[1].atan2(velocity[0]),
            turn_rate: if rng.gen::<bool>() { -1.0 * turn_rate } else { turn_rate },
            mode,
        }
    }).collect()
}
