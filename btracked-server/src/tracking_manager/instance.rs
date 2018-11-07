use std::{collections::HashMap, thread, time::{Duration, Instant}};

use crossbeam_channel as channel;
use serde_json;
use tracking::{
    {Measurement, FilterRunner, TrackingConfig, SignalSource, Particle, filter_runner::ParticleMode},
    distance_field::DistanceField,
};

use super::simulation::{SimulationConfig, SimulationHandle};
use {util, types::BeaconPacket, update_listener::{UpdateListener, ListenerRx}};

#[derive(Clone, Serialize)]
pub struct InstanceConfig {
    pub name: String,
    pub map_key: String,
    pub tracking: TrackingConfig,
    pub beacon_mapping: HashMap<String, String>,
}

#[derive(Clone, Serialize)]
pub struct InstanceDetails {
    pub config: InstanceConfig,
    pub attached_simulations: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct ParticleSnapshot {
    pub stationary: Vec<f32>,
    pub moving: Vec<f32>,
}

impl ParticleSnapshot {
    pub fn new(particles: Vec<Particle>) -> ParticleSnapshot {
        let mut stationary = vec![];
        let mut moving = vec![];

        for particle in particles {
            match particle.mode {
                ParticleMode::Stationary => stationary.extend(&particle.position.xy()),
                ParticleMode::Moving => moving.extend(&particle.position.xy()),
            }
        }

        ParticleSnapshot { stationary, moving }
    }
}

pub struct InstanceHandle {
    pub config: InstanceConfig,
    pub attached_simulations: HashMap<String, SimulationHandle>,
    signal_sources: HashMap<String, SignalSource>,
    command_tx: channel::Sender<InstanceCommand>,
    message_rx: channel::Receiver<InstanceMessage>,
    measurement_listeners: UpdateListener<()>,
    raw_measurement_listeners: UpdateListener<()>,
}

impl InstanceHandle {
    pub fn new(config: InstanceConfig, distance_field: DistanceField) -> InstanceHandle {
        let (command_tx, command_rx) = channel::bounded(0);
        let (message_tx, message_rx) = channel::bounded(0);

        let signal_sources = util::fix_signal_sources(&config.tracking.geometry);

        let config_ = config.clone();
        thread::spawn(move || {
            let instance = Instance::new(config_, distance_field, command_rx, message_tx);
            instance.run()
        });

        InstanceHandle {
            config,
            signal_sources,
            attached_simulations: HashMap::new(),
            command_tx,
            message_rx,

            measurement_listeners: UpdateListener::new(),
            raw_measurement_listeners: UpdateListener::new(),
        }
    }

    pub fn info(&self) -> InstanceDetails {
        InstanceDetails {
            config: self.config.clone(),
            attached_simulations: self.attached_simulations.keys().cloned().collect(),
        }
    }

    pub fn start_sim(&mut self, sim_name: String, sim_config: SimulationConfig) {
        let sim = SimulationHandle::new(
            sim_config,
            self.config.tracking.clone(),
            self.command_tx.clone()
        );
        self.attached_simulations.insert(sim_name, sim);
    }

    pub fn get_sim_mut(&mut self, sim_name: &str) -> Option<&mut SimulationHandle> {
        self.attached_simulations.get_mut(sim_name)
    }

    pub fn stop_sim(&mut self, sim_name: &str) {
        self.attached_simulations.remove(sim_name);
    }

    pub fn add_state_listener(&mut self, config: StateListenerConfig) -> ListenerRx {
        self.command_tx.send(InstanceCommand::AddListener(config));
        match self.message_rx.recv() {
            Some(InstanceMessage::ListenerRx(data)) => data,
            _ => panic!("Invalid response to `AddListener` command"),
        }
    }

    pub fn add_measurement_listener(&mut self, raw: bool) -> ListenerRx {
        match raw {
            true => self.raw_measurement_listeners.add_listener(()),
            false => self.measurement_listeners.add_listener(())
        }
    }

    pub fn get_snapshot(&mut self, sample_size: usize) -> Vec<Particle> {
        self.command_tx.send(InstanceCommand::GetSnapshot(sample_size));
        match self.message_rx.recv() {
            Some(InstanceMessage::Snapshot(data)) => data,
            _ => panic!("Invalid response to `GetSnapshot` command"),
        }
    }

    pub fn get_estimate(&mut self) -> Particle {
        self.command_tx.send(InstanceCommand::GetTarget);
        match self.message_rx.recv() {
            Some(InstanceMessage::Target(data)) => data,
            _ => panic!("Invalid response to `GetTarget` command"),
        }
    }

    pub fn new_measurement(&mut self, measurement: Vec<Measurement>) {
        let measurement_json = serde_json::to_value(&measurement).unwrap();
        self.measurement_listeners.sender(|_| measurement_json.clone());

        self.command_tx.send(InstanceCommand::NewMeasurement(measurement));
    }

    /// Resolves a `BeaconPacket` into a measurement object if the mac matches a known base-station
    pub fn beacon_to_measurement(&mut self, packet: &BeaconPacket) -> Option<Measurement> {
        let measurement_json = serde_json::to_value(packet).unwrap();
        self.raw_measurement_listeners.sender(|_| measurement_json.clone());

        let id = self.config.beacon_mapping.get(&packet.mac).unwrap_or(&packet.mac);
        self.signal_sources.get(id).map(|&source| Measurement { source, rssi: packet.rssi.into() })
    }
}

#[derive(Copy, Clone)]
pub struct StateListenerConfig {
    pub num_particles: usize,
}

pub enum InstanceCommand {
    GetSnapshot(usize),
    GetTarget,
    NewMeasurement(Vec<Measurement>),
    AddListener(StateListenerConfig),
}

pub enum InstanceMessage {
    Snapshot(Vec<Particle>),
    Target(Particle),
    ListenerRx(ListenerRx),
}

pub struct Instance {
    name: String,
    update_rate: u64,
    measurement_buffer: Vec<Measurement>,
    filter_runner: FilterRunner,
    command_rx: channel::Receiver<InstanceCommand>,
    message_tx: channel::Sender<InstanceMessage>,
    update_listener: UpdateListener<StateListenerConfig>,
}

impl Instance {
    pub fn new(
        config: InstanceConfig,
        distance_field: DistanceField,
        command_rx: channel::Receiver<InstanceCommand>,
        message_tx: channel::Sender<InstanceMessage>
    ) -> Instance {
        let filter_runner = FilterRunner::new(&config.tracking, Some(distance_field));
        Instance {
            name: config.name,
            update_rate: config.tracking.filter.update_rate_ms,
            measurement_buffer: vec![],
            filter_runner,
            command_rx,
            message_tx,
            update_listener: UpdateListener::new(),
        }
    }

    pub fn run(mut self) {
        let mut prev_time = Instant::now();
        let process_tick = channel::tick(Duration::from_millis(self.update_rate));

        loop {
            select! {
                recv(self.command_rx, msg) => match msg {
                    Some(cmd) => self.handle_cmd(cmd),
                    None => break,
                },

                recv(process_tick) => {
                    let now = Instant::now();
                    let dt = util::total_seconds(now.duration_since(prev_time)).min(10.0);
                    prev_time = now;

                    self.process(dt);
                }
            }
        }
        info!("Instance: {} thread stopped", self.name);
    }

    fn process(&mut self, dt: f64) {
        let weight = self.filter_runner.step(&*self.measurement_buffer, dt as f32);
        debug!("Processed: {} measurements, dt: {}, weight: {}", self.measurement_buffer.len(), dt, weight);
        self.measurement_buffer.clear();
        self.send_updates();
    }

    fn send_updates(&mut self) {
        fn get_filter_summary(runner: &FilterRunner, num_particles: usize) -> serde_json::Value {
            json!({
                "snapshot": ParticleSnapshot::new(runner.get_snapshot(num_particles)),
                "estimate": runner.get_estimate(),
            })
        }

        let filter_runner = &self.filter_runner;
        self.update_listener.sender(|config| {
            get_filter_summary(filter_runner, config.num_particles)
        });
    }

    fn handle_cmd(&mut self, msg: InstanceCommand) {
        match msg {
            InstanceCommand::GetSnapshot(sample_size) => {
                let particles = self.filter_runner.get_snapshot(sample_size);
                self.message_tx.send(InstanceMessage::Snapshot(particles));
            },
            InstanceCommand::GetTarget => {
                let estimate = self.filter_runner.get_estimate();
                self.message_tx.send(InstanceMessage::Target(estimate));
            },
            InstanceCommand::NewMeasurement(measurement) => {
                self.measurement_buffer.extend(measurement);
            },
            InstanceCommand::AddListener(config) => {
                let listener_rx = self.update_listener.add_listener(config);
                self.message_tx.send(InstanceMessage::ListenerRx(listener_rx));
            }
        }
    }
}
