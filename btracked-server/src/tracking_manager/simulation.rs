use std::{collections::HashMap, thread, time::{Duration, Instant}};

use crossbeam_channel as channel;
use glm;
use rand::{FromEntropy, rngs::SmallRng};
use tracking::{Measurement, MeasurementModel, TrackingConfig, SignalSource, SignalConfig, Particle, filter_runner::ParticleMode};

use super::instance::InstanceCommand;
use util;

pub type State = Particle;

#[derive(Serialize, Deserialize)]
pub struct SimulationConfig {
    pub initial_state: State,
    pub min_rssi: i16,
    pub signal_config: Option<Vec<SignalConfig>>,
}

pub enum SimulationCommand {
    GetState,
    GoTo([f32; 2]),
}

pub enum SimulationMessage {
    State(State),
}

pub struct SimulationHandle {
    command_tx: channel::Sender<SimulationCommand>,
    message_rx: channel::Receiver<SimulationMessage>,
}

impl SimulationHandle {
    pub fn new(
        sim_config: SimulationConfig,
        tracking_config: TrackingConfig,
        measurement_tx: channel::Sender<InstanceCommand>,
    ) -> SimulationHandle {
        let (command_tx, command_rx) = channel::bounded(0);
        let (message_tx, message_rx) = channel::bounded(0);

        thread::spawn(move || {
            let sim = Simulation::new(sim_config, tracking_config, command_rx, message_tx, measurement_tx);
            sim.run()
        });

        SimulationHandle {
            command_tx,
            message_rx,
        }
    }

    pub fn get_state(&mut self) -> State {
        self.command_tx.send(SimulationCommand::GetState);
        match self.message_rx.recv() {
            Some(SimulationMessage::State(data)) => data,
            _ => panic!("Invalid response to `GetState` command"),
        }
    }

    pub fn goto(&mut self, target: [f32; 2]) {
        self.command_tx.send(SimulationCommand::GoTo(target));
    }
}

pub struct Simulation {
    command_rx: channel::Receiver<SimulationCommand>,
    message_tx: channel::Sender<SimulationMessage>,
    measurement_tx: channel::Sender<InstanceCommand>,
    state: State,
    min_rssi: i16,
    max_distance_sqr: f32,
    target: Option<glm::Vec3>,
    measurement_model: MeasurementModel,
    signal_sources: HashMap<String, SignalSource>,
    rng: SmallRng,
}

impl Simulation {
    pub fn new(
        sim_config: SimulationConfig,
        tracking_config: TrackingConfig,
        command_rx: channel::Receiver<SimulationCommand>,
        message_tx: channel::Sender<SimulationMessage>,
        measurement_tx: channel::Sender<InstanceCommand>,
    ) -> Simulation {

        let signal_sources = util::fix_signal_sources(&tracking_config.geometry);
        let measurement_model = match sim_config.signal_config {
            Some(config) => MeasurementModel::new(config),
            None => MeasurementModel::new(tracking_config.filter.signal.clone()),
        };

        let min_rssi = sim_config.min_rssi;
        let max_distance = measurement_model.distance_bound_for_rssi(min_rssi as f32, 100.0);

        info!("Signal model bounds, distance: {}, rssi: {}", max_distance, min_rssi);

        Simulation {
            command_rx,
            message_tx,
            measurement_tx,
            state: sim_config.initial_state,
            min_rssi,
            max_distance_sqr: max_distance * max_distance,
            target: None,
            measurement_model,
            signal_sources,
            rng: SmallRng::from_entropy(),
        }
    }

    /// Runs the simulation continuously until all senders attached to `command_rx` are dropped.
    /// This generally occurs when the corresponding SimulationHandel is dropped.
    pub fn run(mut self) {
        let mut prev_time = Instant::now();
        let update_tick = channel::tick(Duration::from_millis(20));
        let measurement_tick = channel::tick(Duration::from_millis(200));

        loop {
            select! {
                recv(self.command_rx, msg) => match msg {
                    Some(cmd) => self.handle_cmd(cmd),
                    None => break,
                },

                recv(update_tick) => {
                    let now = Instant::now();
                    let dt = util::total_seconds(now.duration_since(prev_time)).min(10.0);
                    prev_time = now;
                    self.update(dt);
                },

                recv(measurement_tick) => {
                    let m = self.generate_measurements();
                    debug!("{} measurements generated", m.len());
                    self.measurement_tx.send(InstanceCommand::NewMeasurement(m));
                },
            }
        }

        debug!("Simulation thread stopped");
    }

    fn update(&mut self, dt: f64) {
        match self.target {
            Some(target) => {
                if glm::distance(&self.state.position, &target) < 0.1 * 0.1 {
                    self.target = None;
                    return;
                }

                let old_pos = self.state.position;
                let direction = (target - old_pos).normalize();

                self.state.velocity = direction * 0.5;
                self.state.position += (dt as f32) * self.state.velocity;
                let delta_p = self.state.position - old_pos;
                self.state.pose = delta_p.y.atan2(delta_p.x);
                self.state.mode = ParticleMode::Moving;
            }
            None => {
                self.state.mode = ParticleMode::Stationary;
            }
        }
    }

    fn generate_measurements(&mut self) -> Vec<Measurement> {
        let state_pos = self.state.position.xy();

        let mut measurements = vec![];
        for (key, source) in &self.signal_sources {
            if glm::distance2(&source.position.xy(), &state_pos) > self.max_distance_sqr {
                continue;
            }

            let measurement = self.measurement_model.generate(&mut self.rng, source, &self.state);
            if measurement.rssi > self.min_rssi {
                let prob = self.measurement_model.probability(&[measurement], self.state);
                trace!("{} -- {:?} {}", key, measurement, prob);
                measurements.push(measurement);
            }
        }
        measurements
    }

    fn handle_cmd(&mut self, msg: SimulationCommand) {
        match msg {
            SimulationCommand::GetState => {
                self.message_tx.send(SimulationMessage::State(self.state.clone()));
            },
            SimulationCommand::GoTo(pos) => {
                self.target = Some(glm::vec3(pos[0], pos[1], 0.0));
            }
        }
    }
}
