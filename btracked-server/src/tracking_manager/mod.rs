pub mod instance;
pub mod simulation;

use std::collections::HashMap;

use parking_lot::Mutex;

use map::distance_field_from_png;
use self::{
    instance::{InstanceConfig, InstanceHandle},
    simulation::{SimulationConfig, SimulationHandle}
};

lazy_static! {
    static ref MANAGER: Mutex<TrackingManager> = Mutex::new(TrackingManager::new());
}

pub fn get_tracking_manager() -> &'static Mutex<TrackingManager> {
    &*MANAGER
}

pub struct TrackingManager {
    instances: HashMap<String, InstanceHandle>,
}

impl TrackingManager {
    pub fn new() -> TrackingManager {
        TrackingManager {
            instances: HashMap::new(),
        }
    }

    pub fn summary(&self) -> Vec<InstanceMetadata> {
        let mut summary = Vec::with_capacity(self.instances.len());
        for (name, instance) in &self.instances {
            summary.push(InstanceMetadata {
                name: name.clone(),
                map_key: instance.config.map_key.clone(),
                attached_simulations: instance.attached_simulations.keys().cloned().collect(),
            });
        }
        summary
    }

    pub fn new_instance(
        &mut self,
        config: InstanceConfig,
        collision_map: &[u8]
    ) -> Result<(), String> {
        let distance_field = distance_field_from_png(collision_map, &config.tracking.geometry)?;

        let name = config.name.clone();
        let instance = InstanceHandle::new(config, distance_field);
        self.instances.insert(name, instance);

        Ok(())
    }

    pub fn stop_instance(&mut self, name: &str) {
        self.instances.remove(name);
    }

    pub fn get_instance_mut(&mut self, name: &str) -> Option<&mut InstanceHandle> {
        self.instances.get_mut(name)
    }

    pub fn new_sim(
        &mut self,
        instance_name: &str,
        sim_name: String,
        config: SimulationConfig,
    ) -> Option<()> {
        self.instances.get_mut(instance_name)?.start_sim(sim_name.clone(), config);
        Some(())
    }

    pub fn stop_sim(&mut self, instance_name: &str, sim_name: &str) {
        if let Some(instance) = self.instances.get_mut(instance_name) {
            instance.stop_sim(sim_name);
        }
    }

    pub fn get_sim_mut(
        &mut self,
        instance_name: &str,
        sim_name: &str
    ) -> Option<&mut SimulationHandle> {
        self.instances.get_mut(instance_name)?.get_sim_mut(sim_name)
    }
}

#[derive(Clone, Serialize)]
pub struct InstanceMetadata {
    pub name: String,
    pub map_key: String,
    pub attached_simulations: Vec<String>,
}