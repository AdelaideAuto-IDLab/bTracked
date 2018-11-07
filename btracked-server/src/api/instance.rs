use std::collections::HashMap;
use actix_web::*;
use futures::Future;

use tracking::{Particle, Measurement, TrackingConfig, FilterConfig};

use {
    AppState, db::map_info::GetMapInfo,
    tracking_manager::{get_tracking_manager, InstanceMetadata},
    tracking_manager::instance::{InstanceConfig, InstanceDetails, InstanceHandle},
    types::BeaconPacket,
};

#[derive(Serialize, Deserialize)]
pub struct NewInstance {
    pub map_key: String,
    pub filter_config: FilterConfig,
    pub beacon_mapping: HashMap<String, String>,
}

pub fn list(_: State<AppState>) -> Json<Vec<InstanceMetadata>> {
    Json(get_tracking_manager().lock().summary())
}

pub fn start((path, body, state): (Path<String>, Json<NewInstance>, State<AppState>))
    -> FutureResponse<HttpResponse>
{
    let name = path.into_inner();
    let NewInstance { map_key, filter_config, beacon_mapping } = body.into_inner();

    state.db.send(GetMapInfo { map_key: map_key.clone() })
        .from_err()
        .and_then(|data| match data {
            Ok((map_geometry, collision_map)) => {
                let config = InstanceConfig {
                    name,
                    map_key,
                    tracking: TrackingConfig { geometry: map_geometry, filter: filter_config },
                    beacon_mapping,
                };

                let mut ctx = get_tracking_manager().lock();
                try_block!(ctx.new_instance(config, &collision_map))?;
                Ok(HttpResponse::Ok().json(true))
            },
            Err(err) => {
                error!("{:?}", err);
                Ok(HttpResponse::InternalServerError().into())
            }
        }).responder()
}

pub fn stop(name: Path<String>) -> Json<bool> {
    get_tracking_manager().lock().stop_instance(&name);
    Json(true)
}

pub fn get_info(name: Path<String>) -> Result<Json<InstanceDetails>> {
    Ok(Json(with_instance(&name, |i| i.info())?))
}

pub fn get_snapshot(path: Path<(String, usize)>) -> Result<Json<Vec<Particle>>> {
let (name, num_particles) = path.into_inner();
    Ok(Json(with_instance(&name, |i| i.get_snapshot(num_particles))?))
}

pub fn get_estimate(name: Path<String>) -> Result<Json<Particle>> {
    Ok(Json(with_instance(&name, |i| i.get_estimate())?))
}

pub fn new_measurement((name, body): (Path<String>, Json<Vec<Measurement>>)) -> Result<Json<bool>> {
    with_instance(&name, |i| i.new_measurement(body.into_inner()))?;
    Ok(Json(true))
}

pub fn new_beacon_measurement((name, body): (Path<String>, Json<Vec<BeaconPacket>>)) -> Result<Json<bool>> {
    with_instance(&name, |i| {
        let measurements = body.into_inner()
            .iter()
            .filter_map(|x| i.beacon_to_measurement(x))
            .collect();
        i.new_measurement(measurements);
    })?;
    Ok(Json(true))
}

fn with_instance<T>(
    instance_name: &str,
    func: impl FnOnce(&mut InstanceHandle) -> T
) -> Result<T> {
    let mut ctx = get_tracking_manager().lock();
    let instance = ctx
        .get_instance_mut(instance_name)
        .ok_or_else(|| error::ErrorNotFound(instance_name.to_owned()))?;
    Ok(func(instance))
}