use std::{collections::HashMap, time::Duration};
use tracking::{GeometryConfig, SignalSource};
use glm;

/// Maps an a Future to a responder where the response can be serialized as a JSON object.
/// Any errors are logged and mapping to `InternalServerError`s to be returned externally.
macro_rules! map_response {
    ($resp:expr) => ($resp.from_err().and_then(|res| match res {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(err) => Ok({
            error!("{:?}", err);
            HttpResponse::InternalServerError().into()
        }),
    }).responder())
}

/// Wraps a block of in a closure to capture any errors and convert them to `InternalServerError`s.
macro_rules! try_block {
    ($val:expr) => (
        (|| -> Result<_, _> { $val })()
            .map_err(|e| e.to_string())
            .map_err(::actix_web::error::ErrorInternalServerError)
    )
}

/// Computes the total number of seconds from a duration
pub fn total_seconds(duration: Duration) -> f64 {
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1e9
}

pub fn fix_signal_sources(config: &GeometryConfig) -> HashMap<String, SignalSource> {
    let mut sources = config.signal_sources.clone();

    let offset = glm::vec3(-config.boundary.x, -config.boundary.y, 0.0);
    let scale = 1.0 / config.scale;

    for source in sources.values_mut() {
        source.position = (source.position + offset) * scale;
    }

    sources
}