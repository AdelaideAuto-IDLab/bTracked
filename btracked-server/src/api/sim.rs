use actix_web::*;

use tracking_manager::{get_tracking_manager, simulation::{SimulationConfig, SimulationHandle}};
use tracking::Particle;

pub fn start((path, body): (Path<(String, String)>, Json<SimulationConfig>)) -> Result<Json<bool>> {
    let (instance_name, sim_name) = path.into_inner();
    match get_tracking_manager().lock().new_sim(&instance_name, sim_name, body.into_inner()) {
        Some(_) => Ok(Json(true)),
        None => Err(error::ErrorNotFound(instance_name)),
    }
}

pub fn stop(path: Path<(String, String)>) -> Json<bool> {
    let (instance_name, sim_name) = path.into_inner();
    get_tracking_manager().lock().stop_sim(&instance_name, &sim_name);
    Json(true)
}

pub fn state(path: Path<(String, String)>) -> Result<Json<Particle>> {
    let (instance_name, sim_name) = path.into_inner();
    Ok(Json(with_sim(&instance_name, &sim_name, |sim| sim.get_state())?))
}

pub fn goto((path, body): (Path<(String, String)>, Json<[f32; 2]>)) -> Result<Json<bool>> {
    let (instance_name, sim_name) = path.into_inner();
    with_sim(&instance_name, &sim_name, |sim| sim.goto(body.into_inner()))?;
    Ok(Json(true))
}

fn with_sim<T>(
    instance_name: &str,
    sim_name: &str,
    func: impl FnOnce(&mut SimulationHandle) -> T
) -> Result<T> {
    let mut ctx = get_tracking_manager().lock();
    let sim = ctx
        .get_sim_mut(instance_name, sim_name)
        .ok_or_else(|| error::ErrorNotFound(sim_name.to_owned()))?;
    Ok(func(sim))
}